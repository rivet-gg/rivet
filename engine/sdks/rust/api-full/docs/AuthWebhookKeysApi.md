# \AuthWebhookKeysApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**auth_webhook_keys_get**](AuthWebhookKeysApi.md#auth_webhook_keys_get) | **GET** /namespaces/{namespace}/auth/webhook-keys | 
[**auth_webhook_keys_revoke**](AuthWebhookKeysApi.md#auth_webhook_keys_revoke) | **POST** /namespaces/{namespace}/auth/webhook-keys/revoke | 
[**auth_webhook_keys_rotate**](AuthWebhookKeysApi.md#auth_webhook_keys_rotate) | **POST** /namespaces/{namespace}/auth/webhook-keys/rotate | 
[**auth_webhooks_ensure**](AuthWebhookKeysApi.md#auth_webhooks_ensure) | **POST** /auth/webhooks/ensure | 



## auth_webhook_keys_get

> models::AuthWebhookKeyStatus auth_webhook_keys_get(namespace)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**namespace** | **String** | Namespace name | [required] |

### Return type

[**models::AuthWebhookKeyStatus**](AuthWebhookKeyStatus.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_webhook_keys_revoke

> models::AuthWebhookKeyStatus auth_webhook_keys_revoke(namespace, auth_webhook_revoke_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**namespace** | **String** | Namespace name | [required] |
**auth_webhook_revoke_request** | [**AuthWebhookRevokeRequest**](AuthWebhookRevokeRequest.md) |  | [required] |

### Return type

[**models::AuthWebhookKeyStatus**](AuthWebhookKeyStatus.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_webhook_keys_rotate

> models::AuthWebhookKeyStatus auth_webhook_keys_rotate(namespace, auth_webhook_rotate_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**namespace** | **String** | Namespace name | [required] |
**auth_webhook_rotate_request** | [**AuthWebhookRotateRequest**](AuthWebhookRotateRequest.md) |  | [required] |

### Return type

[**models::AuthWebhookKeyStatus**](AuthWebhookKeyStatus.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_webhooks_ensure

> models::AuthWebhookMetadata auth_webhooks_ensure()


### Parameters

This endpoint does not need any parameter.

### Return type

[**models::AuthWebhookMetadata**](AuthWebhookMetadata.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

