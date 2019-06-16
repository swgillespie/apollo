package main // import "github.com/swgillespie/apollo/apollod"

import (
	"os"

	log "github.com/sirupsen/logrus"

	"github.com/swgillespie/apollo/apollod/pkg/server"
)

func main() {
	log.SetLevel(log.InfoLevel)
	lichessToken := os.Getenv("LICHESS_TOKEN")
	if lichessToken == "" {
		log.Fatalln("failed to read LICHESS_TOKEN")
	}

	svr, err := server.NewServer(lichessToken)
	if err != nil {
		log.WithError(err).Fatalln("failed to assume lichess account role")
	}

	if err = svr.Run(); err != nil {
		log.WithError(err).Fatalln("failed to launch server")
	}
}

/*
func testUci() {
	log.Info("launching")
	transport, err := NewProgramTransport("/home/swgillespie/.cargo/bin/apollo")
	if err != nil {
		log.WithError(err).Fatalln("failed to launch process")
	}

	log.Info("negotiating UCI")
	client, err := NewUCIClient(transport)
	if err != nil {
		log.WithError(err).Fatalln("failed to negogiate UCI")
	}

	log.WithField("name", client.Name()).Info("client name")
	log.WithField("author", client.Author()).Info("client author")

	if err := client.UCINewGame(); err != nil {
		log.WithError(err).Fatalln("failed to start new game")
	}
	if err := client.IsReady(); err != nil {
		log.WithError(err).Fatalln("failed to ready up")
	}
	if err := client.Position("startpos", nil); err != nil {
		log.WithError(err).Fatalln("failed to set position")
	}
	bestmove, err := client.Go(0, 0, 0, 0)
	if err != nil {
		log.WithError(err).Fatalln("failed to query position")
	}
	log.Info("best move: " + bestmove)

	if err := client.Stop(); err != nil {
		log.WithError(err).Fatalln("failed to stop")
	}
	if err := client.Quit(); err != nil {
		log.WithError(err).Fatalln("failed to quit")
	}
	if err := client.Close(); err != nil {
		log.WithError(err).Fatalln("failed to close client")
	}
}
*/
