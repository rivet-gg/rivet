# \VariablesApi

All URIs are relative to *http://127.0.0.1:4646/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_variable**](VariablesApi.md#delete_variable) | **DELETE** /var/{path} | 
[**get_variable_query**](VariablesApi.md#get_variable_query) | **GET** /var/{path} | 
[**get_variables_list_request**](VariablesApi.md#get_variables_list_request) | **GET** /vars | 
[**post_variable**](VariablesApi.md#post_variable) | **POST** /var/{path} | 
[**put_variable**](VariablesApi.md#put_variable) | **PUT** /var/{path} | 



## delete_variable

> delete_variable(path, variable, region, namespace, x_nomad_token, idempotency_token, cas)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**path** | **String** | A path to a Nomad Variable | [required] |
**variable** | [**Variable**](Variable.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |
**cas** | Option<**i32**> | A compare-and-set parameter for Nomad Variables |  |

### Return type

 (empty response body)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_variable_query

> crate::models::Variable get_variable_query(path, region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**path** | **String** | A path to a Nomad Variable | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**index** | Option<**i32**> | If set, wait until query exceeds given index. Must be provided with WaitParam. |  |
**wait** | Option<**String**> | Provided with IndexParam to wait for change. |  |
**stale** | Option<**String**> | If present, results will include stale reads. |  |
**prefix** | Option<**String**> | Constrains results to jobs that start with the defined prefix |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**per_page** | Option<**i32**> | Maximum number of results to return. |  |
**next_token** | Option<**String**> | Indicates where to start paging for queries that support pagination. |  |

### Return type

[**crate::models::Variable**](Variable.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_variables_list_request

> Vec<crate::models::VariableMetadata> get_variables_list_request(region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**index** | Option<**i32**> | If set, wait until query exceeds given index. Must be provided with WaitParam. |  |
**wait** | Option<**String**> | Provided with IndexParam to wait for change. |  |
**stale** | Option<**String**> | If present, results will include stale reads. |  |
**prefix** | Option<**String**> | Constrains results to jobs that start with the defined prefix |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**per_page** | Option<**i32**> | Maximum number of results to return. |  |
**next_token** | Option<**String**> | Indicates where to start paging for queries that support pagination. |  |

### Return type

[**Vec<crate::models::VariableMetadata>**](VariableMetadata.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_variable

> crate::models::Variable post_variable(path, variable, region, namespace, x_nomad_token, idempotency_token, cas)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**path** | **String** | A path to a Nomad Variable | [required] |
**variable** | [**Variable**](Variable.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |
**cas** | Option<**i32**> | A compare-and-set parameter for Nomad Variables |  |

### Return type

[**crate::models::Variable**](Variable.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_variable

> crate::models::Variable put_variable(path, variable, region, namespace, x_nomad_token, idempotency_token, cas)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**path** | **String** | A path to a Nomad Variable | [required] |
**variable** | [**Variable**](Variable.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |
**cas** | Option<**i32**> | A compare-and-set parameter for Nomad Variables |  |

### Return type

[**crate::models::Variable**](Variable.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

