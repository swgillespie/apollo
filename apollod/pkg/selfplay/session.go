package selfplay

import (
	"context"
	"strconv"
	"sync/atomic"

	"github.com/swgillespie/apollo/apollod/pkg/uci"

	"github.com/notnil/chess"
	log "github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"
)

type Session struct {
	BaselineProgram  string
	CandidateProgram string
	NumGames         int
	NumParallelGames int

	remainingGames int32
	wins           uint32
	losses         uint32
	draws          uint32
}

type Result struct {
	Wins   int
	Losses int
	Draws  int
}

func (s *Session) Run(ctx context.Context) (*Result, error) {
	s.remainingGames = int32(s.NumGames)
	if s.NumParallelGames == 0 {
		s.NumParallelGames = 1
	}

	log.WithField("games", s.remainingGames).Info("beginning selfplay session")
	group, childCtx := errgroup.WithContext(ctx)
	for i := 0; i < s.NumParallelGames; i++ {
		group.Go(func() error {
			return s.worker(i, childCtx)
		})
	}

	log.Info("main thread waiting for workers to complete")
	err := group.Wait()
	log.Info("main thread observed worker completion")
	if err != nil {
		return nil, err
	}

	wins := atomic.LoadUint32(&s.wins)
	losses := atomic.LoadUint32(&s.losses)
	draws := atomic.LoadUint32(&s.draws)
	return &Result{
		Wins:   int(wins),
		Losses: int(losses),
		Draws:  int(draws),
	}, nil
}

func (s *Session) worker(id int, ctx context.Context) error {
	log.WithField("id", id).Info("worker coming online")

	for {
		// Are there remaining games left to be played?
		remainingGames := atomic.AddInt32(&s.remainingGames, -1)
		if remainingGames < 0 {
			// No more games left to play - exit.
			log.WithField("id", id).Info("worker exiting, no remaining games")
			return nil
		}

		log.WithField("id", id).Info("worker playing game")

		// Play a game.
		if err := s.playGame(id, remainingGames%2 == 0); err != nil {
			log.WithError(err).Error("failed to play game")
			return err
		}
	}
}

func (s *Session) playGame(id int, baselineIsWhite bool) error {
	// Load up and initialize our two engines. This launches subprocess for each
	// of the two engines and does the initial UCI handshake for each of them.
	baseline, candidate, err := s.loadEngines()
	if err != nil {
		return err
	}
	defer shutdownEngines(baseline, candidate)

	if baselineIsWhite {
		log.Info("beginning game with baseline as white")
	} else {
		log.Info("beginning game with baseline as black")
	}

	// The baseline and candidate will each play half of their games as black and white.
	var white *uci.Client
	var black *uci.Client
	if baselineIsWhite {
		white = baseline
		black = candidate
	} else {
		white = candidate
		black = baseline
	}

	// Drive the game to completion, using each UCI engine to play white and black.
	notation := chess.LongAlgebraicNotation{}
	game := chess.NewGame(chess.UseNotation(notation))
	whiteToMove := true
	for game.Outcome() == chess.NoOutcome {
		var toMove *uci.Client
		if whiteToMove {
			toMove = white
		} else {
			toMove = black
		}

		// The general UCI procedure here is to send "position startpos" followed
		// by every move that has been played so far, in UCI notation.
		var moves []string
		for _, move := range game.Moves() {
			moves = append(moves, notation.Encode(game.Position(), move))
		}

		if err := toMove.Position("startpos", moves); err != nil {
			return err
		}

		bestmove, err := toMove.Go(0, 0, 0, 0)
		if err != nil {
			return err
		}

		log.WithField("white", strconv.FormatBool(whiteToMove)).Debug("move: " + bestmove)
		moveObj, err := notation.Decode(game.Position(), bestmove)
		if err != nil {
			return err
		}

		if err := game.Move(moveObj); err != nil {
			return err
		}

		whiteToMove = !whiteToMove
	}

	log.WithField("id", id).Info("game completed")

	switch game.Outcome() {
	case chess.Draw:
		log.WithField("id", id).Info("recording draw")
		atomic.AddUint32(&s.draws, 1)
	case chess.WhiteWon:
		if baselineIsWhite {
			log.WithField("id", id).Info("recording loss")
			atomic.AddUint32(&s.losses, 1)
		} else {
			log.WithField("id", id).Info("recording win")
			atomic.AddUint32(&s.wins, 1)
		}
	case chess.BlackWon:
		if baselineIsWhite {
			log.WithField("id", id).Info("recording win")
			atomic.AddUint32(&s.wins, 1)
		} else {
			log.WithField("id", id).Info("recording loss")
			atomic.AddUint32(&s.losses, 1)
		}
	}
	return nil
}

func (s *Session) loadEngines() (*uci.Client, *uci.Client, error) {
	baselineTransport, err := uci.NewProgramTransport(s.BaselineProgram)
	if err != nil {
		return nil, nil, err
	}

	candidateTransport, err := uci.NewProgramTransport(s.CandidateProgram)
	if err != nil {
		return nil, nil, err
	}

	baseline, err := uci.NewClient(baselineTransport)
	if err != nil {
		return nil, nil, err
	}

	candidate, err := uci.NewClient(candidateTransport)
	if err != nil {
		baseline.Close()
		return nil, nil, err
	}

	if err := baseline.UCINewGame(); err != nil {
		baseline.Close()
		candidate.Close()
		return nil, nil, err
	}

	if err := candidate.UCINewGame(); err != nil {
		baseline.Close()
		candidate.Close()
		return nil, nil, err
	}
	return baseline, candidate, nil
}

func shutdownEngines(baseline, candidate *uci.Client) error {
	if err := baseline.Stop(); err != nil {
		return err
	}

	if err := baseline.Quit(); err != nil {
		return err
	}

	if err := candidate.Stop(); err != nil {
		return err
	}

	if err := candidate.Quit(); err != nil {
		return err
	}

	if err := baseline.Close(); err != nil {
		return err
	}
	return candidate.Close()
}
