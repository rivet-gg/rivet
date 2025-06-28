# \ActorsV1MetricsApi

All URIs are relative to *https://api.rivet.gg*

Method | HTTP request | Description
------------- | ------------- | -------------
[**actors_v1_metrics_get**](ActorsV1MetricsApi.md#actors_v1_metrics_get) | **GET** /actors/{actor}/metrics/history | 



## actors_v1_metrics_get

> crate::models::ActorsV1GetActorMetricsResponse actors_v1_metrics_get(actor, start, end, interval, project, environment)


Returns the metrics for a given actor.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor** | **uuid::Uuid** | The id of the actor to destroy | [required] |
**start** | **i32** |  | [required] |
**end** | **i32** |  | [required] |
**interval** | **i32** |  | [required] |
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |

### Return type

[**crate::models::ActorsV1GetActorMetricsResponse**](ActorsV1GetActorMetricsResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

