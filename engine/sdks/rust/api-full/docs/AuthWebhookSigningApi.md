# \AuthWebhookSigningApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**auth_webhooks_sign**](AuthWebhookSigningApi.md#auth_webhooks_sign) | **POST** /auth/webhooks/sign | 



## auth_webhooks_sign

> models::AuthWebhookSignResponse auth_webhooks_sign(body)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**body** | **Vec<u8>** |  | [required] |

### Return type

[**models::AuthWebhookSignResponse**](AuthWebhookSignResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: application/octet-stream
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

