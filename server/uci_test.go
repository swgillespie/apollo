package main

import (
	"io"
	"testing"

	"github.com/stretchr/testify/assert"
)

type MockTransport struct {
	Server func(m *MockTransport, msg string) error

	responses []string
}

func (m *MockTransport) Send(msg string) error {
	return m.Server(m, msg)
}

func (m *MockTransport) Recv() (string, error) {
	if len(m.responses) == 0 {
		return "", io.EOF
	}

	first, rest := m.responses[0], m.responses[1:]
	m.responses = rest
	return first, nil
}

func (m *MockTransport) Respond(msg string) {
	m.responses = append(m.responses, msg)
}

func (m *MockTransport) Close() error { return nil }

func TestUCIHandshake(t *testing.T) {
	trans := &MockTransport{
		Server: func(m *MockTransport, msg string) error {
			assert.Equal(t, msg, "uci")
			m.Respond("id name apollo 0.3.0")
			m.Respond("id author Sean Gillespie <sean@swgillespie.me>")
			m.Respond("uciok")
			return nil
		},
	}

	client, err := NewUCIClient(trans)
	if !assert.NoError(t, err) {
		t.FailNow()
	}
	assert.Equal(t, "apollo 0.3.0", client.Name())
	assert.Equal(t, "Sean Gillespie <sean@swgillespie.me>", client.Author())
}

func TestIsReady(t *testing.T) {
	trans := &MockTransport{
		Server: func(m *MockTransport, msg string) error {
			if msg == "uci" {
				assert.Equal(t, msg, "uci")
				m.Respond("id name apollo 0.3.0")
				m.Respond("id author Sean Gillespie <sean@swgillespie.me>")
				m.Respond("uciok")
				return nil
			}

			assert.Equal(t, msg, "isready")
			m.Respond("readyok")
			return nil
		},
	}

	client, err := NewUCIClient(trans)
	if !assert.NoError(t, err) {
		t.FailNow()
	}
	err = client.IsReady()
	assert.NoError(t, err)
}

func TestUciNewgame(t *testing.T) {
	trans := &MockTransport{
		Server: func(m *MockTransport, msg string) error {
			if msg == "uci" {
				assert.Equal(t, msg, "uci")
				m.Respond("id name apollo 0.3.0")
				m.Respond("id author Sean Gillespie <sean@swgillespie.me>")
				m.Respond("uciok")
				return nil
			}

			assert.Equal(t, msg, "ucinewgame")
			return nil
		},
	}

	client, err := NewUCIClient(trans)
	if !assert.NoError(t, err) {
		t.FailNow()
	}
	err = client.UCINewGame()
	assert.NoError(t, err)
}

func TestPosition(t *testing.T) {
	trans := &MockTransport{
		Server: func(m *MockTransport, msg string) error {
			if msg == "uci" {
				assert.Equal(t, msg, "uci")
				m.Respond("id name apollo 0.3.0")
				m.Respond("id author Sean Gillespie <sean@swgillespie.me>")
				m.Respond("uciok")
				return nil
			}

			assert.Equal(t, msg, "position startpos")
			return nil
		},
	}

	client, err := NewUCIClient(trans)
	if !assert.NoError(t, err) {
		t.FailNow()
	}
	err = client.Position("startpos", nil)
	assert.NoError(t, err)
}

func TestPositionMoves(t *testing.T) {
	trans := &MockTransport{
		Server: func(m *MockTransport, msg string) error {
			if msg == "uci" {
				assert.Equal(t, msg, "uci")
				m.Respond("id name apollo 0.3.0")
				m.Respond("id author Sean Gillespie <sean@swgillespie.me>")
				m.Respond("uciok")
				return nil
			}

			assert.Equal(t, msg, "position startpos moves e2e4 e7e6")
			return nil
		},
	}

	client, err := NewUCIClient(trans)
	if !assert.NoError(t, err) {
		t.FailNow()
	}
	err = client.Position("startpos", []string{"e2e4", "e7e6"})
	assert.NoError(t, err)
}

func TestGo(t *testing.T) {
	trans := &MockTransport{
		Server: func(m *MockTransport, msg string) error {
			if msg == "uci" {
				assert.Equal(t, msg, "uci")
				m.Respond("id name apollo 0.3.0")
				m.Respond("id author Sean Gillespie <sean@swgillespie.me>")
				m.Respond("uciok")
				return nil
			}

			assert.Equal(t, msg, "go wtime 5 winc 0 btime 5 binc 0")
			m.Respond("info bunch of stuff")
			m.Respond("bestmove e2e4")
			return nil
		},
	}

	client, err := NewUCIClient(trans)
	if !assert.NoError(t, err) {
		t.FailNow()
	}
	bestmove, err := client.Go(5, 5, 0, 0)
	assert.NoError(t, err)
	assert.Equal(t, "e2e4", bestmove)
}
