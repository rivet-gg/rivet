# \RoutesApi

All URIs are relative to *https://api.rivet.gg*

Method | HTTP request | Description
------------- | ------------- | -------------
[**routes_delete**](RoutesApi.md#routes_delete) | **DELETE** /routes/{name_id} | 
[**routes_list**](RoutesApi.md#routes_list) | **GET** /routes | 
[**routes_update**](RoutesApi.md#routes_update) | **PUT** /routes/{name_id} | 



## routes_delete

> serde_json::Value routes_delete(name_id, project, environment)


Deletes a route.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name_id** | **String** |  | [required] |
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## routes_list

> crate::models::RoutesListRoutesResponse routes_list(project, environment)


Lists all routes of the given environment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |

### Return type

[**crate::models::RoutesListRoutesResponse**](RoutesListRoutesResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## routes_update

> crate::models::RoutesUpdateRouteResponse routes_update(name_id, routes_update_route_body, project, environment)


Creates or updates a route.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name_id** | **String** |  | [required] |
**routes_update_route_body** | [**RoutesUpdateRouteBody**](RoutesUpdateRouteBody.md) |  | [required] |
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |

### Return type

[**crate::models::RoutesUpdateRouteResponse**](RoutesUpdateRouteResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

