// This file was auto-generated by Fern from our API Definition.

package builds

import (
	bytes "bytes"
	context "context"
	json "encoding/json"
	errors "errors"
	fmt "fmt"
	uuid "github.com/google/uuid"
	io "io"
	http "net/http"
	url "net/url"
	sdk "sdk"
	actor "sdk/actor"
	core "sdk/core"
)

type Client struct {
	baseURL string
	caller  *core.Caller
	header  http.Header
}

func NewClient(opts ...core.ClientOption) *Client {
	options := core.NewClientOptions()
	for _, opt := range opts {
		opt(options)
	}
	return &Client{
		baseURL: options.BaseURL,
		caller:  core.NewCaller(options.HTTPClient),
		header:  options.ToHeader(),
	}
}

// Lists all builds of the game associated with the token used. Can be filtered by tags in the query string.
func (c *Client) Get(ctx context.Context, gameId uuid.UUID, environmentId uuid.UUID, buildId uuid.UUID, request *actor.GetBuildRequest) (*actor.GetBuildResponse, error) {
	baseURL := "https://api.rivet.gg"
	if c.baseURL != "" {
		baseURL = c.baseURL
	}
	endpointURL := fmt.Sprintf(baseURL+"/"+"games/%v/environments/%v/builds/%v", gameId, environmentId, buildId)

	queryParams := make(url.Values)
	if request.TagsJson != nil {
		queryParams.Add("tags_json", fmt.Sprintf("%v", *request.TagsJson))
	}
	if len(queryParams) > 0 {
		endpointURL += "?" + queryParams.Encode()
	}

	errorDecoder := func(statusCode int, body io.Reader) error {
		raw, err := io.ReadAll(body)
		if err != nil {
			return err
		}
		apiError := core.NewAPIError(statusCode, errors.New(string(raw)))
		decoder := json.NewDecoder(bytes.NewReader(raw))
		switch statusCode {
		case 500:
			value := new(sdk.InternalError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 429:
			value := new(sdk.RateLimitError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 403:
			value := new(sdk.ForbiddenError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 408:
			value := new(sdk.UnauthorizedError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 404:
			value := new(sdk.NotFoundError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 400:
			value := new(sdk.BadRequestError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		}
		return apiError
	}

	var response *actor.GetBuildResponse
	if err := c.caller.Call(
		ctx,
		&core.CallParams{
			URL:          endpointURL,
			Method:       http.MethodGet,
			Headers:      c.header,
			Response:     &response,
			ErrorDecoder: errorDecoder,
		},
	); err != nil {
		return nil, err
	}
	return response, nil
}

// Lists all builds of the game associated with the token used. Can be filtered by tags in the query string.
func (c *Client) List(ctx context.Context, gameId uuid.UUID, environmentId uuid.UUID, request *actor.ListBuildsRequest) (*actor.ListBuildsResponse, error) {
	baseURL := "https://api.rivet.gg"
	if c.baseURL != "" {
		baseURL = c.baseURL
	}
	endpointURL := fmt.Sprintf(baseURL+"/"+"games/%v/environments/%v/builds", gameId, environmentId)

	queryParams := make(url.Values)
	if request.TagsJson != nil {
		queryParams.Add("tags_json", fmt.Sprintf("%v", *request.TagsJson))
	}
	if len(queryParams) > 0 {
		endpointURL += "?" + queryParams.Encode()
	}

	errorDecoder := func(statusCode int, body io.Reader) error {
		raw, err := io.ReadAll(body)
		if err != nil {
			return err
		}
		apiError := core.NewAPIError(statusCode, errors.New(string(raw)))
		decoder := json.NewDecoder(bytes.NewReader(raw))
		switch statusCode {
		case 500:
			value := new(sdk.InternalError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 429:
			value := new(sdk.RateLimitError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 403:
			value := new(sdk.ForbiddenError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 408:
			value := new(sdk.UnauthorizedError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 404:
			value := new(sdk.NotFoundError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 400:
			value := new(sdk.BadRequestError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		}
		return apiError
	}

	var response *actor.ListBuildsResponse
	if err := c.caller.Call(
		ctx,
		&core.CallParams{
			URL:          endpointURL,
			Method:       http.MethodGet,
			Headers:      c.header,
			Response:     &response,
			ErrorDecoder: errorDecoder,
		},
	); err != nil {
		return nil, err
	}
	return response, nil
}

func (c *Client) PatchTags(ctx context.Context, gameId uuid.UUID, environmentId uuid.UUID, buildId uuid.UUID, request *actor.PatchBuildTagsRequest) (*actor.PatchBuildTagsResponse, error) {
	baseURL := "https://api.rivet.gg"
	if c.baseURL != "" {
		baseURL = c.baseURL
	}
	endpointURL := fmt.Sprintf(baseURL+"/"+"games/%v/environments/%v/builds/%v/tags", gameId, environmentId, buildId)

	errorDecoder := func(statusCode int, body io.Reader) error {
		raw, err := io.ReadAll(body)
		if err != nil {
			return err
		}
		apiError := core.NewAPIError(statusCode, errors.New(string(raw)))
		decoder := json.NewDecoder(bytes.NewReader(raw))
		switch statusCode {
		case 500:
			value := new(sdk.InternalError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 429:
			value := new(sdk.RateLimitError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 403:
			value := new(sdk.ForbiddenError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 408:
			value := new(sdk.UnauthorizedError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 404:
			value := new(sdk.NotFoundError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 400:
			value := new(sdk.BadRequestError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		}
		return apiError
	}

	var response *actor.PatchBuildTagsResponse
	if err := c.caller.Call(
		ctx,
		&core.CallParams{
			URL:          endpointURL,
			Method:       http.MethodPatch,
			Headers:      c.header,
			Request:      request,
			Response:     &response,
			ErrorDecoder: errorDecoder,
		},
	); err != nil {
		return nil, err
	}
	return response, nil
}

// Creates a new game build for the given game.
func (c *Client) Prepare(ctx context.Context, gameId uuid.UUID, environmentId uuid.UUID, request *actor.CreateBuildRequest) (*actor.CreateBuildResponse, error) {
	baseURL := "https://api.rivet.gg"
	if c.baseURL != "" {
		baseURL = c.baseURL
	}
	endpointURL := fmt.Sprintf(baseURL+"/"+"games/%v/environments/%v/builds/prepare", gameId, environmentId)

	errorDecoder := func(statusCode int, body io.Reader) error {
		raw, err := io.ReadAll(body)
		if err != nil {
			return err
		}
		apiError := core.NewAPIError(statusCode, errors.New(string(raw)))
		decoder := json.NewDecoder(bytes.NewReader(raw))
		switch statusCode {
		case 500:
			value := new(sdk.InternalError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 429:
			value := new(sdk.RateLimitError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 403:
			value := new(sdk.ForbiddenError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 408:
			value := new(sdk.UnauthorizedError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 404:
			value := new(sdk.NotFoundError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 400:
			value := new(sdk.BadRequestError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		}
		return apiError
	}

	var response *actor.CreateBuildResponse
	if err := c.caller.Call(
		ctx,
		&core.CallParams{
			URL:          endpointURL,
			Method:       http.MethodPost,
			Headers:      c.header,
			Request:      request,
			Response:     &response,
			ErrorDecoder: errorDecoder,
		},
	); err != nil {
		return nil, err
	}
	return response, nil
}

// Marks an upload as complete.
func (c *Client) Complete(ctx context.Context, gameId uuid.UUID, environmentId uuid.UUID, buildId uuid.UUID) error {
	baseURL := "https://api.rivet.gg"
	if c.baseURL != "" {
		baseURL = c.baseURL
	}
	endpointURL := fmt.Sprintf(baseURL+"/"+"games/%v/environments/%v/builds/%v/complete", gameId, environmentId, buildId)

	errorDecoder := func(statusCode int, body io.Reader) error {
		raw, err := io.ReadAll(body)
		if err != nil {
			return err
		}
		apiError := core.NewAPIError(statusCode, errors.New(string(raw)))
		decoder := json.NewDecoder(bytes.NewReader(raw))
		switch statusCode {
		case 500:
			value := new(sdk.InternalError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 429:
			value := new(sdk.RateLimitError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 403:
			value := new(sdk.ForbiddenError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 408:
			value := new(sdk.UnauthorizedError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 404:
			value := new(sdk.NotFoundError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		case 400:
			value := new(sdk.BadRequestError)
			value.APIError = apiError
			if err := decoder.Decode(value); err != nil {
				return apiError
			}
			return value
		}
		return apiError
	}

	if err := c.caller.Call(
		ctx,
		&core.CallParams{
			URL:          endpointURL,
			Method:       http.MethodPost,
			Headers:      c.header,
			ErrorDecoder: errorDecoder,
		},
	); err != nil {
		return err
	}
	return nil
}