package blitz

import (
	"bufio"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/url"
)

type Challenger struct {
	ID     string `json:"id"`
	Name   string `json:"name"`
	Title  string `json:"title"`
	Rating int    `json:"rating"`
	Patron bool   `json:"patron"`
	Online bool   `json:"online"`
	Lag    int    `json:"lag"`
}

type DestUser struct {
	ID          string `json:"id"`
	Name        string `json:"name"`
	Title       string `json:"title"`
	Rating      int    `json:"rating"`
	Provisional bool   `json:"provisional"`
	Online      bool   `json:"online"`
	Lag         int    `json:"lag"`
}

type Variant struct {
	Key   string `json:"key"`
	Name  string `json:"name"`
	Short string `json:"short"`
}

type TimeControl struct {
	Type      string `json:"type"`
	Limit     int    `json:"limit"`
	Increment int    `json:"increment"`
	Show      string `json:"show"`
}

type Perf struct {
	Icon string `json:"icon"`
	Name string `json:"name"`
}

type Challenge struct {
	ID          string      `json:"id"`
	Status      string      `json:"status"`
	Challenger  Challenger  `json:"challenger"`
	DestUser    DestUser    `json:"destUser"`
	Variant     Variant     `json:"variant"`
	Rated       bool        `json:"rated"`
	TimeControl TimeControl `json:"timeControl"`
	Color       string      `json:"color"`
	Perf        Perf        `json:"perf"`
}

type GameStart struct {
	ID string `json:"id"`
}

type ChallengeEvent interface {
	challenge()
}

func (c Challenge) challenge()  {}
func (gs GameStart) challenge() {}

type ChallengesService interface {
	StreamEvents(ctx context.Context) (<-chan ChallengeEvent, error)
	AcceptChallenge(ctx context.Context, challengeID string) error
	DeclineChallenge(ctx context.Context, challengeID string) error
}

type challengesServiceImpl struct {
	client *Client
}

func (c *challengesServiceImpl) StreamEvents(ctx context.Context) (<-chan ChallengeEvent, error) {
	stream, err := c.client.stream(ctx, "api/stream/event")
	if err != nil {
		return nil, err
	}

	events := make(chan ChallengeEvent)
	go func() {
		defer close(events)
		defer stream.Close()
		scanner := bufio.NewScanner(stream)
		for scanner.Scan() {
			if scanner.Text() == "" {
				// Lichess periodically sends newlines. Ignore them and move on.
				continue
			}

			payload := make(map[string]json.RawMessage)
			if err := json.Unmarshal([]byte(scanner.Text()), &payload); err != nil {
				return
			}

			var ty string
			if err := json.Unmarshal(payload["type"], &ty); err != nil {
				return
			}

			switch ty {
			case "challenge":
				var challenge Challenge
				if err := json.Unmarshal(payload["challenge"], &challenge); err != nil {
					return
				}
				events <- challenge
			case "gameStart":
				var game GameStart
				if err := json.Unmarshal(payload["game"], &game); err != nil {
					return
				}
				events <- game
			default:
				return
			}
		}
	}()

	return events, nil
}

func (c *challengesServiceImpl) AcceptChallenge(ctx context.Context, challengeID string) error {
	target := fmt.Sprintf("api/challenge/%s/accept", url.PathEscape(challengeID))
	var resp struct {
		Ok bool `json:"ok"`
	}
	if err := c.client.post(ctx, target, nil, &resp); err != nil {
		return err
	}
	if !resp.Ok {
		return errors.New("lichess did not respond with 'ok'")
	}
	return nil
}

func (c *challengesServiceImpl) DeclineChallenge(ctx context.Context, challengeID string) error {
	target := fmt.Sprintf("api/challenge/%s/decline", url.PathEscape(challengeID))
	var resp struct {
		Ok bool `json:"ok"`
	}
	if err := c.client.post(ctx, target, nil, &resp); err != nil {
		return err
	}
	if !resp.Ok {
		return errors.New("lichess did not respond with 'ok'")
	}
	return nil
}
