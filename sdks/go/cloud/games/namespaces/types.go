// This file was auto-generated by Fern from our API Definition.

package namespaces

import (
	json "encoding/json"
	fmt "fmt"
	cloud "sdk/cloud"
	core "sdk/core"
)

type GetGameNamespaceVersionHistoryRequest struct {
	// How many items to offset the search by.
	Anchor *string `json:"-"`
	// Amount of items to return. Must be between 1 and 32 inclusive.
	Limit *int `json:"-"`
}

type InspectResponse struct {
	Agent *cloud.AuthAgent `json:"agent,omitempty"`

	_rawJSON json.RawMessage
}

func (i *InspectResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler InspectResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*i = InspectResponse(value)
	i._rawJSON = json.RawMessage(data)
	return nil
}

func (i *InspectResponse) String() string {
	if len(i._rawJSON) > 0 {
		if value, err := core.StringifyJSON(i._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(i); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", i)
}
