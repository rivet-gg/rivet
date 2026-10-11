package signing_test

import (
	"bytes"
	"context"
	"io"
	"net/http"
	"net/http/httptest"
	"sdk/authwebhooksigning"
	"sdk/option"
	"testing"
)

func TestGeneratedSigningClientPreservesBytes(t *testing.T) {
	for _, body := range [][]byte{{}, {0}, {255, 254, 128}, []byte("\r\n{ \"a\": 1 }\r\n")} {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			actual, err := io.ReadAll(r.Body)
			if err != nil {
				t.Error(err)
			}
			if !bytes.Equal(actual, body) {
				t.Error("body bytes changed")
			}
			if r.URL.Path != "/auth/webhooks/sign" || r.Header.Get("Content-Type") != "application/octet-stream" || r.Header.Get("Authorization") != "Bearer sdk-test" {
				t.Error("unexpected request headers/path")
			}
			w.Header().Set("Content-Type", "application/json")
			io.WriteString(w, `{"timestamp":"123","kid":"ds_test","signature":"test"}`)
		}))
		client := authwebhooksigning.NewClient(option.WithBaseURL(server.URL), option.WithToken("sdk-test"))
		response, err := client.AuthWebhooksSign(context.Background(), body)
		if err != nil {
			t.Fatal(err)
		}
		if response.Timestamp != "123" {
			t.Error("bad response")
		}
		server.Close()
	}
}
