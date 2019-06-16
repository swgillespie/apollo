package blitz

import (
	"context"
	"fmt"
	"net/url"
)

type UserResponse struct {
	ID             string   `json:"id"`
	Username       string   `json:"username"`
	Online         bool     `json:"online"`
	Perfs          Perfs    `json:"perfs"`
	CreatedAt      int64    `json:"createdAt"`
	Profile        Profile  `json:"profile"`
	SeenAt         int64    `json:"seenAt"`
	PlayTime       PlayTime `json:"playTime"`
	URL            string   `json:"url"`
	NbFollowing    int      `json:"nbFollowing"`
	NbFollowers    int      `json:"nbFollowers"`
	CompletionRate int      `json:"completionRate"`
	Count          Count    `json:"count"`
}

type UsersService interface {
	GetUser(ctx context.Context, username string) (*UserResponse, error)
}

type usersServiceImpl struct {
	client *Client
}

func (u *usersServiceImpl) GetUser(ctx context.Context, username string) (*UserResponse, error) {
	var userResp UserResponse
	url := fmt.Sprintf("api/user/%s", url.PathEscape(username))
	if err := u.client.get(ctx, url, &userResp); err != nil {
		return nil, err
	}
	return &userResp, nil
}
