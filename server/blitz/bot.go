package blitz

import (
	"bufio"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/url"
)

type GameState struct {
	Type  string `json:"type"`
	Moves string `json:"moves"`
	Wtime int    `json:"wtime"`
	Btime int    `json:"btime"`
	Winc  int    `json:"winc"`
	Binc  int    `json:"binc"`
}

type GameFull struct {
	Type       string     `json:"type"`
	ID         string     `json:"id"`
	Rated      bool       `json:"rated"`
	Variant    Variant    `json:"variant"`
	Clock      Clock      `json:"clock"`
	Speed      string     `json:"speed"`
	Perf       GamePerf   `json:"perf"`
	CreatedAt  int64      `json:"createdAt"`
	White      GamePlayer `json:"white"`
	Black      GamePlayer `json:"black"`
	InitialFen string     `json:"initialFen"`
	State      GameState  `json:"state"`
}

type Clock struct {
	Initial   int `json:"initial"`
	Increment int `json:"increment"`
}

type GamePerf struct {
	Name string `json:"name"`
}

type GamePlayer struct {
	ID          string `json:"id"`
	Name        string `json:"name"`
	Provisional bool   `json:"provisional"`
	Rating      int    `json:"rating"`
	Title       string `json:"title"`
}

type ChatLine struct {
	Type     string `json:"type"`
	Username string `json:"username"`
	Text     string `json:"text"`
	Room     string `json:"room"`
}

type GameEvent interface {
	gameEvent()
}

func (g GameState) gameEvent() {}
func (g GameFull) gameEvent()  {}
func (g ChatLine) gameEvent()  {}

type BotService interface {
	StreamGameEvents(ctx context.Context, gameID string) (<-chan GameEvent, error)

	MakeMove(ctx context.Context, gameID, move string, offerDraw bool) error
	WriteChat(ctx context.Context, gameID, room, text string) error
	AbortGame(ctx context.Context, gameID string) error
	ResignGame(ctx context.Context, gameID string) error
}

type botServiceImpl struct {
	client *Client
}

func (b *botServiceImpl) StreamGameEvents(ctx context.Context, gameID string) (<-chan GameEvent, error) {
	url := fmt.Sprintf("api/bot/game/stream/%s", url.PathEscape(gameID))
	stream, err := b.client.stream(ctx, url)
	if err != nil {
		return nil, err
	}

	events := make(chan GameEvent)
	go func() {
		defer close(events)
		defer stream.Close()
		scanner := bufio.NewScanner(stream)
		for scanner.Scan() {
			if scanner.Text() == "" {
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
			case "gameFull":
				var game GameFull
				if err := json.Unmarshal([]byte(scanner.Text()), &game); err != nil {
					return
				}
				events <- game
			case "gameState":
				var state GameState
				if err := json.Unmarshal([]byte(scanner.Text()), &state); err != nil {
					return
				}
				events <- state
			case "chatLine":
				var line ChatLine
				if err := json.Unmarshal([]byte(scanner.Text()), &line); err != nil {
					return
				}
				events <- line
			default:
				return
			}
		}
	}()
	return events, nil
}

func (b *botServiceImpl) MakeMove(ctx context.Context, gameID, move string, offerDraw bool) error {
	target := fmt.Sprintf("api/bot/game/%s/move/%s", url.PathEscape(gameID), url.PathEscape(move))
	var resp struct {
		Ok bool `json:"ok"`
	}
	if err := b.client.post(ctx, target, nil, &resp); err != nil {
		return err
	}
	if !resp.Ok {
		return errors.New("lichess did not respond with 'ok'")
	}
	return nil
}

func (b *botServiceImpl) WriteChat(ctx context.Context, gameID, room, text string) error {
	target := fmt.Sprintf("api/bot/game/%s/chat", url.PathEscape(gameID))
	args := map[string]string{
		"room": room,
		"text": text,
	}
	var resp struct {
		Ok bool `json:"ok"`
	}
	if err := b.client.post(ctx, target, args, &resp); err != nil {
		return err
	}
	if !resp.Ok {
		return errors.New("lichess did not respond with 'ok'")
	}
	return nil
}

func (b *botServiceImpl) AbortGame(ctx context.Context, gameID string) error {
	target := fmt.Sprintf("api/bot/game/%s/abort", url.PathEscape(gameID))
	var resp struct {
		Ok bool `json:"ok"`
	}
	if err := b.client.post(ctx, target, nil, &resp); err != nil {
		return err
	}
	if !resp.Ok {
		return errors.New("lichess did not respond with 'ok'")
	}
	return nil
}

func (b *botServiceImpl) ResignGame(ctx context.Context, gameID string) error {
	target := fmt.Sprintf("api/bot/game/%s/resign", url.PathEscape(gameID))
	var resp struct {
		Ok bool `json:"ok"`
	}
	if err := b.client.post(ctx, target, nil, &resp); err != nil {
		return err
	}
	if !resp.Ok {
		return errors.New("lichess did not respond with 'ok'")
	}
	return nil
}
