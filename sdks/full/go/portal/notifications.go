// This file was auto-generated by Fern from our API Definition.

package portal

import (
	json "encoding/json"
	fmt "fmt"
	core "sdk/core"
)

type NotificationUnregisterService string

const (
	NotificationUnregisterServiceFirebase NotificationUnregisterService = "firebase"
)

func NewNotificationUnregisterServiceFromString(s string) (NotificationUnregisterService, error) {
	switch s {
	case "firebase":
		return NotificationUnregisterServiceFirebase, nil
	}
	var t NotificationUnregisterService
	return "", fmt.Errorf("%s is not a valid %T", s, t)
}

func (n NotificationUnregisterService) Ptr() *NotificationUnregisterService {
	return &n
}

type RegisterNotificationsRequest struct {
	Service *NotificationRegisterService `json:"service,omitempty"`

	_rawJSON json.RawMessage
}

func (r *RegisterNotificationsRequest) UnmarshalJSON(data []byte) error {
	type unmarshaler RegisterNotificationsRequest
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*r = RegisterNotificationsRequest(value)
	r._rawJSON = json.RawMessage(data)
	return nil
}

func (r *RegisterNotificationsRequest) String() string {
	if len(r._rawJSON) > 0 {
		if value, err := core.StringifyJSON(r._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(r); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", r)
}

type UnregisterNotificationsRequest struct {
	// Represents a value for which notification service to unregister.
	Service NotificationUnregisterService `json:"-"`
}