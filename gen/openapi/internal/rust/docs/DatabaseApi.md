# \DatabaseApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**database_delete**](DatabaseApi.md#database_delete) | **POST** /collections/{collection}/delete | 
[**database_fetch**](DatabaseApi.md#database_fetch) | **POST** /collections/{collection}/fetch | 
[**database_insert**](DatabaseApi.md#database_insert) | **POST** /collections/{collection}/insert | 
[**database_update**](DatabaseApi.md#database_update) | **POST** /collections/{collection}/update | 



## database_delete

> crate::models::DatabaseDeleteResponse database_delete(collection, database_delete_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**collection** | **String** |  | [required] |
**database_delete_request** | [**DatabaseDeleteRequest**](DatabaseDeleteRequest.md) |  | [required] |

### Return type

[**crate::models::DatabaseDeleteResponse**](DatabaseDeleteResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## database_fetch

> crate::models::DatabaseFetchResponse database_fetch(collection, database_fetch_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**collection** | **String** |  | [required] |
**database_fetch_request** | [**DatabaseFetchRequest**](DatabaseFetchRequest.md) |  | [required] |

### Return type

[**crate::models::DatabaseFetchResponse**](DatabaseFetchResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## database_insert

> crate::models::DatabaseInsertResponse database_insert(collection, database_insert_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**collection** | **String** |  | [required] |
**database_insert_request** | [**DatabaseInsertRequest**](DatabaseInsertRequest.md) |  | [required] |

### Return type

[**crate::models::DatabaseInsertResponse**](DatabaseInsertResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## database_update

> crate::models::DatabaseUpdateResponse database_update(collection, database_update_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**collection** | **String** |  | [required] |
**database_update_request** | [**DatabaseUpdateRequest**](DatabaseUpdateRequest.md) |  | [required] |

### Return type

[**crate::models::DatabaseUpdateResponse**](DatabaseUpdateResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

