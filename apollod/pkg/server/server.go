package server

import (
	"context"
	"os/exec"
	"strconv"
	"strings"

	"github.com/pkg/errors"
	log "github.com/sirupsen/logrus"
	"golang.org/x/sync/semaphore"

	"github.com/swgillespie/apollo/apollod/pkg/blitz"
	"github.com/swgillespie/apollo/apollod/pkg/uci"
)

const (
	maxPendingChallenges = 3
	maxConcurrentGames   = 1
)

type Server struct {
	client        *blitz.Client
	challenges    chan blitz.Challenge
	gameSemaphore *semaphore.Weighted
}

func NewServer(token string) (*Server, error) {
	client := blitz.New(token)
	user, err := client.Account.GetProfile(context.Background())
	if err != nil {
		return nil, errors.Wrap(err, "failed to read lichess profile")
	}

	// Only proceed if we're using a bot account - these are special in lichess.
	log.WithField("username", user.Username).Infoln("authenticated with lichess")
	if user.Title != "BOT" {
		log.WithField("username", user.Username).Warningln("user is not a BOT")
		return nil, errors.New("specified user is not a bot")
	}

	return &Server{
		client:        client,
		challenges:    make(chan blitz.Challenge, maxPendingChallenges),
		gameSemaphore: semaphore.NewWeighted(maxConcurrentGames),
	}, nil
}

func (s *Server) Run() error {
	ctx := context.Background()
	events, err := s.client.Challenges.StreamEvents(ctx)
	if err != nil {
		return errors.Wrap(err, "failed to read lichess event stream")
	}

	go s.challengeLoop()
	log.Infoln("server waiting for incoming events")
	for event := range events {
		switch e := event.(type) {
		case blitz.Challenge:
			if err := s.HandleChallenge(ctx, e); err != nil {
				log.WithError(err).Error("failed to accept challenge")
			}
		case blitz.GameStart:
			s.HandleGameStart(ctx, e)
		}
	}

	return nil
}

func (s *Server) HandleChallenge(ctx context.Context, challenge blitz.Challenge) error {
	log.WithFields(log.Fields{
		"challenger": challenge.Challenger.Name,
		"rating":     challenge.Challenger.Rating,
		"game":       challenge.Variant,
		"id":         challenge.ID,
	}).Infoln("received challenge")

	select {
	case s.challenges <- challenge:
		log.WithField("id", challenge.ID).
			Infoln("enqueued challenge")
	default:
		log.WithField("id", challenge.ID).
			Infoln("too many pending challenges, declining challenge")
		return s.client.Challenges.DeclineChallenge(ctx, challenge.ID)
	}
	return nil
}

func (s *Server) challengeLoop() {
	ctx := context.Background()
	log.Info("challenge loop starting")
	for challenge := range s.challenges {
		if !apolloPlaysVariant(challenge.Variant) {
			log.WithField("variant", challenge.Variant.Key).Info("declining challenge, apollo does not play this variant")
			if err := s.client.Challenges.DeclineChallenge(ctx, challenge.ID); err != nil {
				log.WithError(err).Info("failed to decline challenge")
			}
			continue
		}

		log.WithField("id", challenge.ID).Info("accepting challenge")
		if err := s.client.Challenges.AcceptChallenge(ctx, challenge.ID); err != nil {
			log.WithError(err).Info("failed to accept challenge")
			continue
		}
	}
}

func (s *Server) HandleGameStart(ctx context.Context, gameStart blitz.GameStart) {
	// defer s.gameSemaphore.Release(1)
	log.WithField("id", gameStart.ID).Info("beginning game")
	if err := s.playGame(ctx, gameStart); err != nil {
		log.WithError(err).Error("fatal error while playing game")
		if err := s.client.Bot.AbortGame(ctx, gameStart.ID); err != nil {
			log.WithError(err).Info("failed to abort game")
			if err := s.client.Bot.ResignGame(ctx, gameStart.ID); err != nil {
				log.WithError(err).Error("failed to resign game")
			}
		}
	}
}

func (s *Server) playGame(ctx context.Context, gameStart blitz.GameStart) error {
	// Lichess directs us to switch APIs as soon as we get GameStart. We'll now start streaming
	// events for that particular game.
	//
	// First, though, we need to fire up Apollo.
	client, err := loadAndInitializeApollo()
	if err != nil {
		return err
	}

	// Next, we need to do tell Apollo to start a new game.
	if err := client.UCINewGame(); err != nil {
		return err
	}

	// Be friendly?
	if err := s.client.Bot.WriteChat(ctx, gameStart.ID, "player", "Good Luck, Have Fun! Check me out on GitHub at https://github.com/swgillespie/apollo"); err != nil {
		log.WithError(err).Warning("failed to send friendly chat message")
	}

	// Lichess is going to stream us events for this game. Get the stream and iterate over it.
	stream, err := s.client.Bot.StreamGameEvents(ctx, gameStart.ID)
	if err != nil {
		return err
	}

	// Keep track of who's turn it is. Lichess will slap us if we play out of turn and it's our
	// job to figure out when to play.
	ourTurn := false

	// Lichess also sends us a GameState event for our own moves, so we need to skip those too.
	nextIsOurOwnMove := false

	for event := range stream {
		var bestmove string
		switch e := event.(type) {
		case blitz.GameFull:
			log.Info("received GameFull event")
			ourTurn = apolloIsWhite(e)
			log.WithField("isWhite", strconv.FormatBool(ourTurn)).Info("determining which side apollo play on")
			log.WithField("moves", e.State.Moves).Debug("incoming moves")

			// Sometimes, if our opponent is VERY fast, the first GameFull contains the first move
			// that our opponent did. If it's not our turn (i.e. we're black), it may be the case
			// that our opponent has already played a move.
			if e.State.Moves != "" {
				log.Info("our opponent was super fast and played a move already, it is our turn despite being black")
				ourTurn = !ourTurn
			}

			if !ourTurn {
				log.Info("skipping state and not playing, not our turn")
				ourTurn = !ourTurn
				continue
			}

			nextIsOurOwnMove = true
			move, err := engineEvaluate(client, e.State)
			if err != nil {
				return err
			}
			bestmove = move
		case blitz.GameState:
			log.Info("received GameState event")
			if nextIsOurOwnMove {
				log.Info("skipping state and not playing, this is our own move")
				nextIsOurOwnMove = !nextIsOurOwnMove
				continue
			}

			if !ourTurn {
				log.Info("skipping state and not playing, not our turn")
				ourTurn = !ourTurn
				continue
			}

			nextIsOurOwnMove = true
			move, err := engineEvaluate(client, e)
			if err != nil {
				return err
			}
			bestmove = move
		case blitz.ChatLine:
			// Ignore, don't care.
			continue
		}

		log.WithField("move", bestmove).Info("sending move to lichess")
		if err := s.client.Bot.MakeMove(ctx, gameStart.ID, bestmove, false); err != nil {
			return err
		}
	}

	log.Info("stream has ended, completing game")
	return nil
}

func engineEvaluate(client *uci.Client, state blitz.GameState) (string, error) {
	moves := strings.Split(state.Moves, " ")
	if err := client.Position("startpos", moves); err != nil {
		return "", err
	}

	bestmove, err := client.Go(state.Wtime, state.Btime, state.Winc, state.Binc)
	if err != nil {
		return "", err
	}
	return bestmove, nil
}

func loadAndInitializeApollo() (*uci.Client, error) {
	// Loading up Apollo entails launching apollo as a subprocess, hooking up our stdin and
	// stdout accordingly, and then performing the base UCI handshake.
	//
	// If there's an apollo on the path, use that, otherwise use an adjacent apollo.
	apolloFromPath, err := exec.LookPath("apollo")
	if err != nil {
		apolloFromPath = "./apollo"
	}

	transport, err := uci.NewProgramTransport(apolloFromPath)
	if err != nil {
		return nil, err
	}

	return uci.NewClient(transport)
}

// apolloIsWhite returns true if Apollo is the white player in this game, false otherwise.
func apolloIsWhite(fullGame blitz.GameFull) bool {
	return fullGame.White.ID == "apollo_bot"
}

// apolloPlaysVariant returns true if Apollo can play the requested chess variant. Lichess supports a bunch of variants
// that Apollo doesn't know how to play.
func apolloPlaysVariant(variant blitz.Variant) bool {
	switch variant.Key {
	case "standard", "ultraBullet", "bullet", "blitz", "rapid", "classical", "correspondence":
		return true
	default:
		return false
	}
}
