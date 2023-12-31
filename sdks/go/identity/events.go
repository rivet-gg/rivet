// This file was auto-generated by Fern from our API Definition.

package identity

import (
	json "encoding/json"
	fmt "fmt"
	sdk "sdk"
	core "sdk/core"
)

type WatchEventsResponse struct {
	Events []*GlobalEvent     `json:"events,omitempty"`
	Watch  *sdk.WatchResponse `json:"watch,omitempty"`

	_rawJSON json.RawMessage
}

func (w *WatchEventsResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler WatchEventsResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*w = WatchEventsResponse(value)
	w._rawJSON = json.RawMessage(data)
	return nil
}

func (w *WatchEventsResponse) String() string {
	if len(w._rawJSON) > 0 {
		if value, err := core.StringifyJSON(w._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(w); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", w)
}

type WatchEventsRequest struct {
	WatchIndex sdk.WatchQuery `json:"-"`
}
