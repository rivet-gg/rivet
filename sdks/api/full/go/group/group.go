// This file was auto-generated by Fern from our API Definition.

package group

import (
	json "encoding/json"
	fmt "fmt"
	uuid "github.com/google/uuid"
	sdk "sdk"
	group "sdk/common/group"
	core "sdk/core"
	upload "sdk/upload"
)

type CreateRequest struct {
	DisplayName sdk.DisplayName `json:"display_name"`

	_rawJSON json.RawMessage
}

func (c *CreateRequest) UnmarshalJSON(data []byte) error {
	type unmarshaler CreateRequest
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*c = CreateRequest(value)
	c._rawJSON = json.RawMessage(data)
	return nil
}

func (c *CreateRequest) String() string {
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

type CreateResponse struct {
	GroupId uuid.UUID `json:"group_id"`

	_rawJSON json.RawMessage
}

func (c *CreateResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler CreateResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*c = CreateResponse(value)
	c._rawJSON = json.RawMessage(data)
	return nil
}

func (c *CreateResponse) String() string {
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

type GetBansResponse struct {
	// A list of banned group members.
	BannedIdentities []*BannedIdentity `json:"banned_identities,omitempty"`
	// The pagination anchor.
	Anchor *string            `json:"anchor,omitempty"`
	Watch  *sdk.WatchResponse `json:"watch,omitempty"`

	_rawJSON json.RawMessage
}

func (g *GetBansResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler GetBansResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*g = GetBansResponse(value)
	g._rawJSON = json.RawMessage(data)
	return nil
}

func (g *GetBansResponse) String() string {
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

type GetJoinRequestsResponse struct {
	// A list of group join requests.
	JoinRequests []*JoinRequest `json:"join_requests,omitempty"`
	// The pagination anchor.
	Anchor *string            `json:"anchor,omitempty"`
	Watch  *sdk.WatchResponse `json:"watch,omitempty"`

	_rawJSON json.RawMessage
}

func (g *GetJoinRequestsResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler GetJoinRequestsResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*g = GetJoinRequestsResponse(value)
	g._rawJSON = json.RawMessage(data)
	return nil
}

func (g *GetJoinRequestsResponse) String() string {
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

type GetMembersResponse struct {
	// A list of group members.
	Members []*Member `json:"members,omitempty"`
	// The pagination anchor.
	Anchor *string            `json:"anchor,omitempty"`
	Watch  *sdk.WatchResponse `json:"watch,omitempty"`

	_rawJSON json.RawMessage
}

func (g *GetMembersResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler GetMembersResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*g = GetMembersResponse(value)
	g._rawJSON = json.RawMessage(data)
	return nil
}

func (g *GetMembersResponse) String() string {
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

type GetProfileResponse struct {
	Group *Profile           `json:"group,omitempty"`
	Watch *sdk.WatchResponse `json:"watch,omitempty"`

	_rawJSON json.RawMessage
}

func (g *GetProfileResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler GetProfileResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*g = GetProfileResponse(value)
	g._rawJSON = json.RawMessage(data)
	return nil
}

func (g *GetProfileResponse) String() string {
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

type GetSummaryResponse struct {
	Group *group.GroupSummary `json:"group,omitempty"`

	_rawJSON json.RawMessage
}

func (g *GetSummaryResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler GetSummaryResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*g = GetSummaryResponse(value)
	g._rawJSON = json.RawMessage(data)
	return nil
}

func (g *GetSummaryResponse) String() string {
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

type ListSuggestedResponse struct {
	// A list of group summaries.
	Groups []*group.GroupSummary `json:"groups,omitempty"`
	Watch  *sdk.WatchResponse    `json:"watch,omitempty"`

	_rawJSON json.RawMessage
}

func (l *ListSuggestedResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler ListSuggestedResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*l = ListSuggestedResponse(value)
	l._rawJSON = json.RawMessage(data)
	return nil
}

func (l *ListSuggestedResponse) String() string {
	if len(l._rawJSON) > 0 {
		if value, err := core.StringifyJSON(l._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(l); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", l)
}

type PrepareAvatarUploadRequest struct {
	// The path/filename of the group avatar.
	Path string `json:"path"`
	// The MIME type of the group avatar.
	Mime *string `json:"mime,omitempty"`
	// Unsigned 64 bit integer.
	ContentLength int64 `json:"content_length"`

	_rawJSON json.RawMessage
}

func (p *PrepareAvatarUploadRequest) UnmarshalJSON(data []byte) error {
	type unmarshaler PrepareAvatarUploadRequest
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*p = PrepareAvatarUploadRequest(value)
	p._rawJSON = json.RawMessage(data)
	return nil
}

func (p *PrepareAvatarUploadRequest) String() string {
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

type PrepareAvatarUploadResponse struct {
	UploadId         uuid.UUID                `json:"upload_id"`
	PresignedRequest *upload.PresignedRequest `json:"presigned_request,omitempty"`

	_rawJSON json.RawMessage
}

func (p *PrepareAvatarUploadResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler PrepareAvatarUploadResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*p = PrepareAvatarUploadResponse(value)
	p._rawJSON = json.RawMessage(data)
	return nil
}

func (p *PrepareAvatarUploadResponse) String() string {
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

type TransferOwnershipRequest struct {
	// Identity to transfer the group to.
	// Must be a member of the group.
	NewOwnerIdentityId string `json:"new_owner_identity_id"`

	_rawJSON json.RawMessage
}

func (t *TransferOwnershipRequest) UnmarshalJSON(data []byte) error {
	type unmarshaler TransferOwnershipRequest
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*t = TransferOwnershipRequest(value)
	t._rawJSON = json.RawMessage(data)
	return nil
}

func (t *TransferOwnershipRequest) String() string {
	if len(t._rawJSON) > 0 {
		if value, err := core.StringifyJSON(t._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(t); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", t)
}

type UpdateProfileRequest struct {
	DisplayName *sdk.DisplayName `json:"display_name,omitempty"`
	// Detailed information about a profile.
	Bio       *string          `json:"bio,omitempty"`
	Publicity *group.Publicity `json:"publicity,omitempty"`

	_rawJSON json.RawMessage
}

func (u *UpdateProfileRequest) UnmarshalJSON(data []byte) error {
	type unmarshaler UpdateProfileRequest
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*u = UpdateProfileRequest(value)
	u._rawJSON = json.RawMessage(data)
	return nil
}

func (u *UpdateProfileRequest) String() string {
	if len(u._rawJSON) > 0 {
		if value, err := core.StringifyJSON(u._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(u); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", u)
}

type ValidateProfileRequest struct {
	DisplayName *sdk.DisplayName `json:"display_name,omitempty"`
	Bio         *sdk.DisplayName `json:"bio,omitempty"`
	Publicity   *group.Publicity `json:"publicity,omitempty"`

	_rawJSON json.RawMessage
}

func (v *ValidateProfileRequest) UnmarshalJSON(data []byte) error {
	type unmarshaler ValidateProfileRequest
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*v = ValidateProfileRequest(value)
	v._rawJSON = json.RawMessage(data)
	return nil
}

func (v *ValidateProfileRequest) String() string {
	if len(v._rawJSON) > 0 {
		if value, err := core.StringifyJSON(v._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(v); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", v)
}

type ValidateProfileResponse struct {
	// A list of validation errors.
	Errors []*sdk.ValidationError `json:"errors,omitempty"`

	_rawJSON json.RawMessage
}

func (v *ValidateProfileResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler ValidateProfileResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*v = ValidateProfileResponse(value)
	v._rawJSON = json.RawMessage(data)
	return nil
}

func (v *ValidateProfileResponse) String() string {
	if len(v._rawJSON) > 0 {
		if value, err := core.StringifyJSON(v._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(v); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", v)
}
