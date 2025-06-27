# \ActorsMetricsApi

All URIs are relative to *https://api.rivet.gg*

Method | HTTP request | Description
------------- | ------------- | -------------
[**actors_metrics_get**](ActorsMetricsApi.md#actors_metrics_get) | **GET** /actors/metrics/history | 



## actors_metrics_get

> crate::models::ActorsGetActorMetricsResponse actors_metrics_get(start, end, interval, actor_ids_json, metrics_json, project, environment)


Returns the metrics for a given actor.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**start** | **i32** |  | [required] |
**end** | **i32** |  | [required] |
**interval** | **i32** |  | [required] |
**actor_ids_json** | **String** |  | [required] |
**metrics_json** | **String** |  | [required] |
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |

### Return type

[**crate::models::ActorsGetActorMetricsResponse**](ActorsGetActorMetricsResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

