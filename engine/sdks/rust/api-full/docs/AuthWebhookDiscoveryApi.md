# \AuthWebhookDiscoveryApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**auth_webhooks_jwks**](AuthWebhookDiscoveryApi.md#auth_webhooks_jwks) | **GET** /namespaces/{namespace}/auth/webhooks/jwks.json | 
[**auth_webhooks_metadata**](AuthWebhookDiscoveryApi.md#auth_webhooks_metadata) | **GET** /namespaces/{namespace}/auth/webhooks | 



## auth_webhooks_jwks

> models::AuthWebhookJwks auth_webhooks_jwks(namespace)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**namespace** | **String** | Immutable namespace ID | [required] |

### Return type

[**models::AuthWebhookJwks**](AuthWebhookJwks.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/jwk-set+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_webhooks_metadata

> models::AuthWebhookMetadata auth_webhooks_metadata(namespace)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**namespace** | **String** | Immutable namespace ID | [required] |

### Return type

[**models::AuthWebhookMetadata**](AuthWebhookMetadata.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

