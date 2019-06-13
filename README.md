# Apollo chess engine

Apollo is a small chess engine, written in Rust, accompanied with a small
Lichess-compatible server written in Go. Apollo is a UCI-compatible engine
and is able to speak to any UCI-compatible chess GUI. The author uses PyChess
mostly to play with the engine.

Using the server component, Apollo is able to play on Lichess. The Lichess bot
operated by me plays under [apollo_bot](https://lichess.org/@/apollo_bot) moniker.
You can challenge it on Lichess!

## Design

Apollo is an extremely standard, unimaginative chess engine. Today, it uses naive
mode ordering combined with a transposition table backed alpha-beta search.
Apollo's search averages a branching factor between 8 and 12, which is not good.
As a result, Apollo can't search very deep.

Apollo needs a lot of work before it plays strong chess, but it can beat beginner
players and bots on Lichess that deliberately lose.

## Server

`apollo-server` is a small Go server that bridges the Lichess REST API and Apollo,
using UCI to communicate with Apollo. This works reasonably well, well enough that
Apollo can play pretty much anybody on Lichess without the server getting confused.

The server makes no attempt to control the number of games that it plays
concurrently. Please be nice to my bot, playing lots of games will set my server
on fire.

## Building and running tests

Apollo requires that you have Go and Rust installed. `make` will compile the engine
and server components, while `make test` will run all tests and `make docker` will
build and tag a Docker image suitable for deployment.