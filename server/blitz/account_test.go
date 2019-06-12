package blitz

import (
	"bytes"
	"context"
	"io/ioutil"
	"net/http"
	"testing"

	"github.com/stretchr/testify/assert"
)

type RoundTripFunc func(req *http.Request) *http.Response

func (f RoundTripFunc) RoundTrip(req *http.Request) (*http.Response, error) {
	return f(req), nil
}

func NewTestClient(fn RoundTripFunc) *http.Client {
	return &http.Client{
		Transport: RoundTripFunc(fn),
	}
}

const profileJSONResult = `
{
    "id": "swgillespie",
    "username": "swgillespie",
    "online": true,
    "perfs": {
        "blitz": {
            "games": 0,
            "rating": 1500,
            "rd": 350,
            "prog": 0,
            "prov": true
        },
        "bullet": {
            "games": 0,
            "rating": 1500,
            "rd": 350,
            "prog": 0,
            "prov": true
        },
        "correspondence": {
            "games": 0,
            "rating": 1500,
            "rd": 350,
            "prog": 0,
            "prov": true
        },
        "puzzle": {
            "games": 216,
            "rating": 1595,
            "rd": 61,
            "prog": 42
        },
        "classical": {
            "games": 0,
            "rating": 1500,
            "rd": 350,
            "prog": 0,
            "prov": true
        },
        "rapid": {
            "games": 12,
            "rating": 1143,
            "rd": 102,
            "prog": 28
        }
    },
    "createdAt": 1552526315654,
    "profile": {
        "country": "US",
        "firstName": "Sean",
        "lastName": "Gillespie"
    },
    "seenAt": 1559927798681,
    "playTime": {
        "total": 33067,
        "tv": 0
    },
    "url": "https://lichess.org/@/swgillespie",
    "nbFollowing": 0,
    "nbFollowers": 0,
    "completionRate": 100,
    "count": {
        "all": 92,
        "rated": 12,
        "ai": 80,
        "draw": 4,
        "drawH": 0,
        "loss": 70,
        "lossH": 7,
        "win": 18,
        "winH": 5,
        "bookmark": 0,
        "playing": 0,
        "import": 0,
        "me": 0
    },
    "followable": true,
    "following": false,
    "blocking": false,
    "followsYou": false
}
`

func TestGetProfile(t *testing.T) {
	httpClient := NewTestClient(func(req *http.Request) *http.Response {
		assert.Equal(t, req.URL.String(), defaultBaseURL+"api/account")
		return &http.Response{
			StatusCode: 200,
			Body:       ioutil.NopCloser(bytes.NewBufferString(profileJSONResult)),
			Header:     make(http.Header),
		}
	})

	client := New("", WithHTTPClient(httpClient))
	resp, err := client.Account.GetProfile(context.Background())
	assert.NoError(t, err)
	assert.Equal(t, resp.ID, "swgillespie")
}
