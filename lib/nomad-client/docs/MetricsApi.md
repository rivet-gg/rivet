# \MetricsApi

All URIs are relative to *http://127.0.0.1:4646/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_metrics_summary**](MetricsApi.md#get_metrics_summary) | **GET** /metrics | 



## get_metrics_summary

> crate::models::MetricsSummary get_metrics_summary(format)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**format** | Option<**String**> | The format the user requested for the metrics summary (e.g. prometheus) |  |

### Return type

[**crate::models::MetricsSummary**](MetricsSummary.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

