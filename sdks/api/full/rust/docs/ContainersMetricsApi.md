# \ContainersMetricsApi

All URIs are relative to *https://api.rivet.gg*

Method | HTTP request | Description
------------- | ------------- | -------------
[**containers_metrics_get**](ContainersMetricsApi.md#containers_metrics_get) | **GET** /v1/containers/{container}/metrics/history | 



## containers_metrics_get

> crate::models::ContainersGetContainerMetricsResponse containers_metrics_get(container, start, end, interval, project, environment)


Returns the metrics for a given container.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**container** | **String** | The id of the container to destroy | [required] |
**start** | **i32** |  | [required] |
**end** | **i32** |  | [required] |
**interval** | **i32** |  | [required] |
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |

### Return type

[**crate::models::ContainersGetContainerMetricsResponse**](ContainersGetContainerMetricsResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

