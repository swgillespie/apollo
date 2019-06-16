package uci

import (
	"bufio"
	"fmt"
	"io"
	"os"
	"os/exec"
	"regexp"
	"strings"

	"github.com/pkg/errors"
	log "github.com/sirupsen/logrus"
)

var (
	idNameRegex   = regexp.MustCompile(`id name (.*)`)
	idAuthorRegex = regexp.MustCompile(`id author (.*)`)
	optionRegex   = regexp.MustCompile(`option (.*)`)
	uciOkRegex    = regexp.MustCompile(`uciok`)
	readyOkRegex  = regexp.MustCompile(`readyok`)
	bestmoveRegex = regexp.MustCompile(`bestmove (.*)`)
)

type Transport interface {
	io.Closer

	Send(msg string) error
	Recv() (string, error)
}

type popenTransport struct {
	process *exec.Cmd
	in      io.Writer
	out     *bufio.Scanner
}

func (p *popenTransport) Close() error {
	return p.process.Wait()
}

func (p *popenTransport) Send(msg string) error {
	_, err := p.in.Write([]byte(msg + "\n"))
	return err
}

func (p *popenTransport) Recv() (string, error) {
	if !p.out.Scan() {
		return "", io.EOF
	}

	return p.out.Text(), nil
}

func (p *popenTransport) LogStderr() error {
	stderr, err := p.process.StderrPipe()
	if err != nil {
		return err
	}

	go func() {
		scanner := bufio.NewScanner(stderr)
		for scanner.Scan() {
			log.WithField("source", "apollo").Info(scanner.Text())
		}
	}()
	return nil
}

func NewProgramTransport(programPath string) (Transport, error) {
	log.WithField("program", programPath).Info("launching new program")
	cmd := exec.Command(programPath)
	cmd.Env = append(os.Environ(), "RUST_LOG=info")
	stdin, err := cmd.StdinPipe()
	if err != nil {
		return nil, err
	}
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return nil, err
	}

	trans := &popenTransport{
		process: cmd,
		in:      stdin,
		out:     bufio.NewScanner(stdout),
	}

	trans.LogStderr()
	if err := cmd.Start(); err != nil {
		return nil, err
	}
	return trans, nil
}

// Client is a wrapper over an input and output stream that speaks the UCI protocol.
// The intention is to use UCI client alongside a UCI-compliant server to instruct the server to search for moves and
// otherwise play the game of chess.
//
// See http://wbec-ridderkerk.nl/html/UCIProtocol.html for details on the protocol itself.
type Client struct {
	transport Transport

	name   string
	author string
}

func NewClient(transport Transport) (*Client, error) {
	client := &Client{
		transport: transport,
		name:      "",
		author:    "",
	}

	if err := client.uci(); err != nil {
		client.Close()
		return nil, err
	}
	return client, nil
}

func (u *Client) Name() string   { return u.name }
func (u *Client) Author() string { return u.author }

func (u *Client) uci() error {
	if err := u.transport.Send("uci"); err != nil {
		return err
	}

	// In response, the server will send a bunch of stuff:
	//  * "id name", identifying the engine by name
	//  * "id author", identifying the author by name
	//  * "option", telling us what options the server supports
	//  * "uciok", telling us that there will be no further messages.
	for {
		line, err := u.transport.Recv()
		if err != nil {
			return err
		}

		switch {
		case idNameRegex.MatchString(line):
			u.name = idNameRegex.FindStringSubmatch(line)[1]
		case idAuthorRegex.MatchString(line):
			u.author = idAuthorRegex.FindStringSubmatch(line)[1]
		case optionRegex.MatchString(line):
			// pass, nothing interesting to do here for now. Apollo doesn't
			// send this.
		case uciOkRegex.MatchString(line):
			return nil
		default:
			// Apollo doesn't send anything other than these.
			return errors.Errorf("unexpected 'uci' response: %s", line)
		}
	}
}

func (u *Client) IsReady() error {
	if err := u.transport.Send("isready"); err != nil {
		return err
	}

	line, err := u.transport.Recv()
	if err != nil {
		return err
	}

	if line != "readyok" {
		return errors.Errorf("unexpected 'isready' response: %s", line)
	}
	return nil
}

func (u *Client) UCINewGame() error {
	return u.transport.Send("ucinewgame")
}

func (u *Client) Position(position string, moves []string) error {
	var command string
	if len(moves) > 0 {
		command = fmt.Sprintf("position %s moves %s", position, strings.Join(moves, " "))
	} else {
		command = fmt.Sprintf("position %s", position)
	}

	return u.transport.Send(command)
}

func (u *Client) Go(wtime, btime, winc, binc int) (string, error) {
	command := fmt.Sprintf("go wtime %d winc %d btime %d binc %d", wtime, winc, btime, binc)
	if err := u.transport.Send(command); err != nil {
		return "", err
	}

	// In response, the server will begin sending a BUNCH of stuff, most of which we don't care about.
	// We care about "bestmove", since this is the engine telling us what move it makes.
	for {
		line, err := u.transport.Recv()
		if err != nil {
			return "", err
		}

		switch {
		case bestmoveRegex.MatchString(line):
			move := bestmoveRegex.FindStringSubmatch(line)[1]
			return move, nil
		default:
			// Roll with anything that's not bestmove.
		}
	}
}

func (u *Client) Stop() error {
	return u.transport.Send("stop")
}

func (u *Client) Quit() error {
	return u.transport.Send("quit")
}

func (u *Client) Close() error {
	return u.transport.Close()
}
