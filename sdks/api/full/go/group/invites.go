// This file was auto-generated by Fern from our API Definition.

package group

import (
	json "encoding/json"
	fmt "fmt"
	uuid "github.com/google/uuid"
	group "sdk/common/group"
	core "sdk/core"
)

type ConsumeInviteResponse struct {
	GroupId *uuid.UUID `json:"group_id,omitempty"`

	_rawJSON json.RawMessage
}

func (c *ConsumeInviteResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler ConsumeInviteResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*c = ConsumeInviteResponse(value)
	c._rawJSON = json.RawMessage(data)
	return nil
}

func (c *ConsumeInviteResponse) String() string {
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

type CreateInviteRequest struct {
	// How long until the group invite expires (in milliseconds).
	Ttl *float64 `json:"ttl,omitempty"`
	// How many times the group invite can be used.
	UseCount *float64 `json:"use_count,omitempty"`

	_rawJSON json.RawMessage
}

func (c *CreateInviteRequest) UnmarshalJSON(data []byte) error {
	type unmarshaler CreateInviteRequest
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*c = CreateInviteRequest(value)
	c._rawJSON = json.RawMessage(data)
	return nil
}

func (c *CreateInviteRequest) String() string {
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

type CreateInviteResponse struct {
	// The code that will be passed to `rivet.api.group#ConsumeInvite` to join a group.
	Code string `json:"code"`

	_rawJSON json.RawMessage
}

func (c *CreateInviteResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler CreateInviteResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*c = CreateInviteResponse(value)
	c._rawJSON = json.RawMessage(data)
	return nil
}

func (c *CreateInviteResponse) String() string {
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

type GetInviteResponse struct {
	Group *group.Handle `json:"group,omitempty"`

	_rawJSON json.RawMessage
}

func (g *GetInviteResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler GetInviteResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*g = GetInviteResponse(value)
	g._rawJSON = json.RawMessage(data)
	return nil
}

func (g *GetInviteResponse) String() string {
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