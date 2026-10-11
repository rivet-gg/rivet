# \AuthIssuersApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**auth_issuers_delete**](AuthIssuersApi.md#auth_issuers_delete) | **DELETE** /namespaces/{namespace}/auth/issuers/{issuer_name} | 
[**auth_issuers_get**](AuthIssuersApi.md#auth_issuers_get) | **GET** /namespaces/{namespace}/auth/issuers/{issuer_name} | 
[**auth_issuers_list**](AuthIssuersApi.md#auth_issuers_list) | **GET** /namespaces/{namespace}/auth/issuers | 
[**auth_issuers_upsert**](AuthIssuersApi.md#auth_issuers_upsert) | **PUT** /namespaces/{namespace}/auth/issuers/{issuer_name} | 



## auth_issuers_delete

> serde_json::Value auth_issuers_delete(namespace, issuer_name)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**namespace** | **String** | Namespace name | [required] |
**issuer_name** | **String** | Issuer registration name | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_issuers_get

> models::AuthIssuerGetResponse auth_issuers_get(namespace, issuer_name)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**namespace** | **String** | Namespace name | [required] |
**issuer_name** | **String** | Issuer registration name | [required] |

### Return type

[**models::AuthIssuerGetResponse**](AuthIssuerGetResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_issuers_list

> models::AuthIssuerListResponse auth_issuers_list(namespace)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**namespace** | **String** | Namespace name | [required] |

### Return type

[**models::AuthIssuerListResponse**](AuthIssuerListResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## auth_issuers_upsert

> models::AuthIssuerUpsertResponse auth_issuers_upsert(namespace, issuer_name, auth_issuer_upsert_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**namespace** | **String** | Namespace name | [required] |
**issuer_name** | **String** | Issuer registration name | [required] |
**auth_issuer_upsert_request** | [**AuthIssuerUpsertRequest**](AuthIssuerUpsertRequest.md) |  | [required] |

### Return type

[**models::AuthIssuerUpsertResponse**](AuthIssuerUpsertResponse.md)

### Authorization

[bearer_auth](../README.md#bearer_auth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

