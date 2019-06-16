package blitz

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"io/ioutil"
	"net/http"
	"net/http/httputil"
	"net/url"

	"github.com/pkg/errors"
	log "github.com/sirupsen/logrus"
)

const (
	defaultBaseURL = "https://lichess.org/"
)

type Client struct {
	baseURL   string
	token     string
	userAgent string
	client    *http.Client

	Account    AccountService
	Users      UsersService
	Challenges ChallengesService
	Bot        BotService
}

type ClientOption func(*Client)

func New(token string, options ...ClientOption) *Client {
	client := &Client{
		baseURL:   defaultBaseURL,
		token:     token,
		userAgent: "Apollo-Blitz/1.0",
		client:    &http.Client{},
	}
	for _, option := range options {
		option(client)
	}

	client.Account = &accountServiceImpl{client}
	client.Users = &usersServiceImpl{client}
	client.Challenges = &challengesServiceImpl{client}
	client.Bot = &botServiceImpl{client}
	return client
}

func WithHTTPClient(httpClient *http.Client) ClientOption {
	return func(client *Client) {
		client.client = httpClient
	}
}

func WithBaseURL(url string) ClientOption {
	return func(client *Client) {
		client.baseURL = url
	}
}

func WithUserAgent(userAgent string) ClientOption {
	return func(client *Client) {
		client.userAgent = userAgent
	}
}

func (c *Client) urlFor(endpoint string) string {
	return c.baseURL + endpoint
}

func (c *Client) get(ctx context.Context, endpoint string, response interface{}) error {
	req, err := http.NewRequest(http.MethodGet, c.urlFor(endpoint), nil)
	if err != nil {
		return err
	}
	if c.userAgent != "" {
		req.Header.Add("User-Agent", c.userAgent)
	}
	req.Header.Add("Authorization", "Bearer "+c.token)
	resp, err := c.client.Do(req.WithContext(ctx))
	if err != nil {
		return err
	}

	defer resp.Body.Close()
	decoder := json.NewDecoder(resp.Body)
	if resp.StatusCode >= 400 {
		var errResponse lichessWireError
		if err := decoder.Decode(&errResponse); err != nil {
			return err
		}

		return LichessError{
			StatusCode: resp.StatusCode,
			Message:    errResponse.Error,
		}
	}

	return decoder.Decode(response)
}

func (c *Client) post(ctx context.Context, endpoint string, args map[string]string, response interface{}) error {
	data := make(url.Values)
	if args != nil {
		for key, value := range args {
			data.Add(key, value)
		}
	}

	dataStr := data.Encode()
	req, err := http.NewRequest(http.MethodPost, c.urlFor(endpoint), bytes.NewBufferString(dataStr))
	if err != nil {
		return errors.Wrap(err, "while creating request")
	}
	if c.userAgent != "" {
		req.Header.Add("User-Agent", c.userAgent)
	}
	req.Header.Add("Authorization", "Bearer "+c.token)
	req.Header.Add("Content-Type", "application/x-www-form-urlencoded")

	if log.IsLevelEnabled(log.DebugLevel) {
		dumped, err := httputil.DumpRequestOut(req, true)
		if err == nil {
			log.Debug(string(dumped))
		}
	}

	resp, err := c.client.Do(req.WithContext(ctx))
	if err != nil {
		return errors.Wrap(err, "while executing request")
	}

	if log.IsLevelEnabled(log.DebugLevel) {
		dumped, err := httputil.DumpResponse(resp, true)
		if err == nil {
			log.Debug(string(dumped))
		}
	}

	defer resp.Body.Close()
	decoder := json.NewDecoder(resp.Body)
	if resp.StatusCode >= 400 {
		if resp.Header.Get("Content-Type") != "application/json" {
			body, err := ioutil.ReadAll(resp.Body)
			if err != nil {
				return errors.Errorf("[%d] lichess responded with failure", resp.StatusCode)
			}

			return errors.Errorf("[%d] %s", resp.StatusCode, body)
		}

		var errResponse lichessWireError
		if err := decoder.Decode(&errResponse); err != nil {
			return errors.Wrap(err, "while decoding error response")
		}

		return LichessError{
			StatusCode: resp.StatusCode,
			Message:    errResponse.Error,
		}
	}

	if err := decoder.Decode(response); err != nil {
		return errors.Wrap(err, "while decoding response")
	}
	return nil
}

func (c *Client) stream(ctx context.Context, endpoint string) (io.ReadCloser, error) {
	req, err := http.NewRequest(http.MethodGet, c.urlFor(endpoint), nil)
	if err != nil {
		return nil, err
	}
	if c.userAgent != "" {
		req.Header.Add("User-Agent", c.userAgent)
	}
	req.Header.Add("Authorization", "Bearer "+c.token)
	resp, err := c.client.Do(req.WithContext(ctx))
	if err != nil {
		return nil, err
	}

	if resp.StatusCode >= 400 {
		defer resp.Body.Close()
		decoder := json.NewDecoder(resp.Body)
		decoder.DisallowUnknownFields()
		var errResponse lichessWireError
		if err := decoder.Decode(&errResponse); err != nil {
			return nil, err
		}

		return nil, LichessError{
			StatusCode: resp.StatusCode,
			Message:    errResponse.Error,
		}
	}
	return resp.Body, nil
}

type lichessWireError struct {
	Error string `json:"error"`
}

type LichessError struct {
	StatusCode int
	Message    string
}

func (l LichessError) Error() string {
	return fmt.Sprintf("[%d] %s", l.StatusCode, l.Message)
}
