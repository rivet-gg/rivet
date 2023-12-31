// This file was auto-generated by Fern from our API Definition.

package devices

import (
	json "encoding/json"
	fmt "fmt"
	uuid "github.com/google/uuid"
	sdk "sdk"
	core "sdk/core"
)

type GetDeviceLinkRequest struct {
	DeviceLinkToken sdk.Jwt        `json:"-"`
	WatchIndex      sdk.WatchQuery `json:"-"`
}

type CompleteDeviceLinkRequest struct {
	DeviceLinkToken sdk.Jwt   `json:"device_link_token"`
	GameId          uuid.UUID `json:"game_id"`

	_rawJSON json.RawMessage
}

func (c *CompleteDeviceLinkRequest) UnmarshalJSON(data []byte) error {
	type unmarshaler CompleteDeviceLinkRequest
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*c = CompleteDeviceLinkRequest(value)
	c._rawJSON = json.RawMessage(data)
	return nil
}

func (c *CompleteDeviceLinkRequest) String() string {
	if len(c._rawJSON) > 0 {
		if value, err := core.StringifyJSON(c._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(c); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", c)
}

type GetDeviceLinkResponse struct {
	CloudToken *string            `json:"cloud_token,omitempty"`
	Watch      *sdk.WatchResponse `json:"watch,omitempty"`

	_rawJSON json.RawMessage
}

func (g *GetDeviceLinkResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler GetDeviceLinkResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*g = GetDeviceLinkResponse(value)
	g._rawJSON = json.RawMessage(data)
	return nil
}

func (g *GetDeviceLinkResponse) String() string {
	if len(g._rawJSON) > 0 {
		if value, err := core.StringifyJSON(g._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(g); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", g)
}

type PrepareDeviceLinkResponse struct {
	DeviceLinkId    uuid.UUID `json:"device_link_id"`
	DeviceLinkToken string    `json:"device_link_token"`
	DeviceLinkUrl   string    `json:"device_link_url"`

	_rawJSON json.RawMessage
}

func (p *PrepareDeviceLinkResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler PrepareDeviceLinkResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*p = PrepareDeviceLinkResponse(value)
	p._rawJSON = json.RawMessage(data)
	return nil
}

func (p *PrepareDeviceLinkResponse) String() string {
	if len(p._rawJSON) > 0 {
		if value, err := core.StringifyJSON(p._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(p); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", p)
}
