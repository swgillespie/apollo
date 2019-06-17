package main // import "github.com/swgillespie/apollo/apollod"

import (
	"context"
	"flag"
	"fmt"
	"os"
	"runtime"

	log "github.com/sirupsen/logrus"

	"github.com/swgillespie/apollo/apollod/pkg/selfplay"
	"github.com/swgillespie/apollo/apollod/pkg/server"
)

var doSelfplay = flag.Bool("selfplay", false, "Run in selfplay mode")
var baselineEngine = flag.String("baseline", "", "Path to baseline selfplay engine")
var candidateEngine = flag.String("candidate", "", "Path to candidate selfplay engine")
var numGames = flag.Int("numGames", 40, "Number of games to play")
var parallelGames = flag.Int("parallel", runtime.NumCPU(), "Number of games to play in parallel")
var debug = flag.Bool("debug", false, "Enable debug logging")

func main() {
	flag.Parse()
	log.SetLevel(log.InfoLevel)
	if *debug {
		log.SetLevel(log.DebugLevel)
	}

	if *doSelfplay {
		runSelfplay()
		return
	}

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

func runSelfplay() {
	session := &selfplay.Session{
		BaselineProgram:  *baselineEngine,
		CandidateProgram: *candidateEngine,
		NumGames:         *numGames,
		NumParallelGames: *parallelGames,
	}

	if session.BaselineProgram == "" {
		log.Fatalln("baseline engine not provided")
	}
	if session.CandidateProgram == "" {
		log.Fatalln("candidate engine not provided")
	}
	ctx := context.Background()
	res, err := session.Run(ctx)
	if err != nil {
		log.WithError(err).Fatalln("failed to run selfplay")
	}

	candidateScore := float64(res.Wins)
	baselineScore := float64(res.Losses)
	candidateScore += float64(res.Draws) / float64(2)
	baselineScore += float64(res.Draws) / float64(2)
	fmt.Printf("final score: %f-%f\n", candidateScore, baselineScore)
}
