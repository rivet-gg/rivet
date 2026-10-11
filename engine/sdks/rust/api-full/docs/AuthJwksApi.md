# \AuthJwksApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**auth_jwks_get**](AuthJwksApi.md#auth_jwks_get) | **GET** /.well-known/jwks.json | Publishes only the public halves of Rivet's currently valid signing keys.



## auth_jwks_get

> serde_json::Value auth_jwks_get()
Publishes only the public halves of Rivet's currently valid signing keys.

### Parameters

This endpoint does not need any parameter.

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

