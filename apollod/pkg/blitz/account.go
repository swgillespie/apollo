package blitz

import "context"

type AccountResponse struct {
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
	Title          string   `json:"title"`
	Followable     bool     `json:"followable"`
	Following      bool     `json:"following"`
	Blocking       bool     `json:"blocking"`
	FollowsYou     bool     `json:"followsYou"`
}

type Blitz struct {
	Games  int  `json:"games"`
	Rating int  `json:"rating"`
	Rd     int  `json:"rd"`
	Prog   int  `json:"prog"`
	Prov   bool `json:"prov"`
}

type Bullet struct {
	Games  int  `json:"games"`
	Rating int  `json:"rating"`
	Rd     int  `json:"rd"`
	Prog   int  `json:"prog"`
	Prov   bool `json:"prov"`
}

type Correspondence struct {
	Games  int  `json:"games"`
	Rating int  `json:"rating"`
	Rd     int  `json:"rd"`
	Prog   int  `json:"prog"`
	Prov   bool `json:"prov"`
}

type Puzzle struct {
	Games  int  `json:"games"`
	Rating int  `json:"rating"`
	Rd     int  `json:"rd"`
	Prog   int  `json:"prog"`
	Prov   bool `json:"prov"`
}

type Classical struct {
	Games  int  `json:"games"`
	Rating int  `json:"rating"`
	Rd     int  `json:"rd"`
	Prog   int  `json:"prog"`
	Prov   bool `json:"prov"`
}

type Rapid struct {
	Games  int  `json:"games"`
	Rating int  `json:"rating"`
	Rd     int  `json:"rd"`
	Prog   int  `json:"prog"`
	Prov   bool `json:"prov"`
}

type Perfs struct {
	Blitz          Blitz          `json:"blitz"`
	Bullet         Bullet         `json:"bullet"`
	Correspondence Correspondence `json:"correspondence"`
	Puzzle         Puzzle         `json:"puzzle"`
	Classical      Classical      `json:"classical"`
	Rapid          Rapid          `json:"rapid"`
}

type Profile struct {
	Country   string `json:"country"`
	FirstName string `json:"firstName"`
	LastName  string `json:"lastName"`
}

type PlayTime struct {
	Total int `json:"total"`
	Tv    int `json:"tv"`
}

type Count struct {
	All      int `json:"all"`
	Rated    int `json:"rated"`
	Ai       int `json:"ai"`
	Draw     int `json:"draw"`
	DrawH    int `json:"drawH"`
	Loss     int `json:"loss"`
	LossH    int `json:"lossH"`
	Win      int `json:"win"`
	WinH     int `json:"winH"`
	Bookmark int `json:"bookmark"`
	Playing  int `json:"playing"`
	Import   int `json:"import"`
	Me       int `json:"me"`
}

type PreferencesResponse struct {
	Prefs Preferences `json:"prefs"`
}
type Preferences struct {
	Animation     int    `json:"animation"`
	AutoQueen     int    `json:"autoQueen"`
	AutoThreefold int    `json:"autoThreefold"`
	BgImg         string `json:"bgImg"`
	Blindfold     int    `json:"blindfold"`
	Captured      bool   `json:"captured"`
	Challenge     int    `json:"challenge"`
	ClockBar      bool   `json:"clockBar"`
	ClockSound    bool   `json:"clockSound"`
	ClockTenths   int    `json:"clockTenths"`
	ConfirmResign int    `json:"confirmResign"`
	CoordColor    int    `json:"coordColor"`
	Coords        int    `json:"coords"`
	Dark          bool   `json:"dark"`
	Destination   bool   `json:"destination"`
	Follow        bool   `json:"follow"`
	Highlight     bool   `json:"highlight"`
	InsightShare  int    `json:"insightShare"`
	Is3D          bool   `json:"is3d"`
	KeyboardMove  int    `json:"keyboardMove"`
	Message       int    `json:"message"`
	MoveEvent     int    `json:"moveEvent"`
	PieceSet      string `json:"pieceSet"`
	PieceSet3D    string `json:"pieceSet3d"`
	Premove       bool   `json:"premove"`
	Replay        int    `json:"replay"`
	SoundSet      string `json:"soundSet"`
	SubmitMove    int    `json:"submitMove"`
	Takeback      int    `json:"takeback"`
	Theme         string `json:"theme"`
	Theme3D       string `json:"theme3d"`
	Transp        bool   `json:"transp"`
	Zen           int    `json:"zen"`
}

type AccountService interface {
	GetProfile(ctx context.Context) (*AccountResponse, error)
	GetEmail(ctx context.Context) (string, error)
	GetPreferences(ctx context.Context) (*PreferencesResponse, error)
}

type accountServiceImpl struct {
	client *Client
}

func (a *accountServiceImpl) GetProfile(ctx context.Context) (*AccountResponse, error) {
	var accountResp AccountResponse
	if err := a.client.get(ctx, "api/account", &accountResp); err != nil {
		return nil, err
	}
	return &accountResp, nil
}

func (a *accountServiceImpl) GetEmail(ctx context.Context) (string, error) {
	var emailResponse struct {
		Email string `json:"email"`
	}
	if err := a.client.get(ctx, "api/account/email", &emailResponse); err != nil {
		return "", err
	}
	return emailResponse.Email, nil
}

func (a *accountServiceImpl) GetPreferences(ctx context.Context) (*PreferencesResponse, error) {
	var prefsResp PreferencesResponse
	if err := a.client.get(ctx, "api/account/preferences", &prefsResp); err != nil {
		return nil, err
	}
	return &prefsResp, nil
}
