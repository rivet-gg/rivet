# \JobsApi

All URIs are relative to *http://127.0.0.1:4646/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_job**](JobsApi.md#delete_job) | **DELETE** /job/{jobName} | 
[**get_job**](JobsApi.md#get_job) | **GET** /job/{jobName} | 
[**get_job_allocations**](JobsApi.md#get_job_allocations) | **GET** /job/{jobName}/allocations | 
[**get_job_deployment**](JobsApi.md#get_job_deployment) | **GET** /job/{jobName}/deployment | 
[**get_job_deployments**](JobsApi.md#get_job_deployments) | **GET** /job/{jobName}/deployments | 
[**get_job_evaluations**](JobsApi.md#get_job_evaluations) | **GET** /job/{jobName}/evaluations | 
[**get_job_scale_status**](JobsApi.md#get_job_scale_status) | **GET** /job/{jobName}/scale | 
[**get_job_summary**](JobsApi.md#get_job_summary) | **GET** /job/{jobName}/summary | 
[**get_job_versions**](JobsApi.md#get_job_versions) | **GET** /job/{jobName}/versions | 
[**get_jobs**](JobsApi.md#get_jobs) | **GET** /jobs | 
[**post_job**](JobsApi.md#post_job) | **POST** /job/{jobName} | 
[**post_job_dispatch**](JobsApi.md#post_job_dispatch) | **POST** /job/{jobName}/dispatch | 
[**post_job_evaluate**](JobsApi.md#post_job_evaluate) | **POST** /job/{jobName}/evaluate | 
[**post_job_parse**](JobsApi.md#post_job_parse) | **POST** /jobs/parse | 
[**post_job_periodic_force**](JobsApi.md#post_job_periodic_force) | **POST** /job/{jobName}/periodic/force | 
[**post_job_plan**](JobsApi.md#post_job_plan) | **POST** /job/{jobName}/plan | 
[**post_job_revert**](JobsApi.md#post_job_revert) | **POST** /job/{jobName}/revert | 
[**post_job_scaling_request**](JobsApi.md#post_job_scaling_request) | **POST** /job/{jobName}/scale | 
[**post_job_stability**](JobsApi.md#post_job_stability) | **POST** /job/{jobName}/stable | 
[**post_job_validate_request**](JobsApi.md#post_job_validate_request) | **POST** /validate/job | 
[**register_job**](JobsApi.md#register_job) | **POST** /jobs | 



## delete_job

> crate::models::JobDeregisterResponse delete_job(job_name, region, namespace, x_nomad_token, idempotency_token, purge, global)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |
**purge** | Option<**bool**> | Boolean flag indicating whether to purge allocations of the job after deleting. |  |
**global** | Option<**bool**> | Boolean flag indicating whether the operation should apply to all instances of the job globally. |  |

### Return type

[**crate::models::JobDeregisterResponse**](JobDeregisterResponse.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job

> crate::models::Job get_job(job_name, region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
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

[**crate::models::Job**](Job.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_allocations

> Vec<crate::models::AllocationListStub> get_job_allocations(job_name, region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token, all)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**index** | Option<**i32**> | If set, wait until query exceeds given index. Must be provided with WaitParam. |  |
**wait** | Option<**String**> | Provided with IndexParam to wait for change. |  |
**stale** | Option<**String**> | If present, results will include stale reads. |  |
**prefix** | Option<**String**> | Constrains results to jobs that start with the defined prefix |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**per_page** | Option<**i32**> | Maximum number of results to return. |  |
**next_token** | Option<**String**> | Indicates where to start paging for queries that support pagination. |  |
**all** | Option<**bool**> | Specifies whether the list of allocations should include allocations from a previously registered job with the same ID. This is possible if the job is deregistered and reregistered. |  |

### Return type

[**Vec<crate::models::AllocationListStub>**](AllocationListStub.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_deployment

> crate::models::Deployment get_job_deployment(job_name, region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
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

[**crate::models::Deployment**](Deployment.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_deployments

> Vec<crate::models::Deployment> get_job_deployments(job_name, region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token, all)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**index** | Option<**i32**> | If set, wait until query exceeds given index. Must be provided with WaitParam. |  |
**wait** | Option<**String**> | Provided with IndexParam to wait for change. |  |
**stale** | Option<**String**> | If present, results will include stale reads. |  |
**prefix** | Option<**String**> | Constrains results to jobs that start with the defined prefix |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**per_page** | Option<**i32**> | Maximum number of results to return. |  |
**next_token** | Option<**String**> | Indicates where to start paging for queries that support pagination. |  |
**all** | Option<**i32**> | Flag indicating whether to constrain by job creation index or not. |  |

### Return type

[**Vec<crate::models::Deployment>**](Deployment.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_evaluations

> Vec<crate::models::Evaluation> get_job_evaluations(job_name, region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
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

[**Vec<crate::models::Evaluation>**](Evaluation.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_scale_status

> crate::models::JobScaleStatusResponse get_job_scale_status(job_name, region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
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

[**crate::models::JobScaleStatusResponse**](JobScaleStatusResponse.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_summary

> crate::models::JobSummary get_job_summary(job_name, region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
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

[**crate::models::JobSummary**](JobSummary.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_versions

> crate::models::JobVersionsResponse get_job_versions(job_name, region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token, diffs)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**index** | Option<**i32**> | If set, wait until query exceeds given index. Must be provided with WaitParam. |  |
**wait** | Option<**String**> | Provided with IndexParam to wait for change. |  |
**stale** | Option<**String**> | If present, results will include stale reads. |  |
**prefix** | Option<**String**> | Constrains results to jobs that start with the defined prefix |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**per_page** | Option<**i32**> | Maximum number of results to return. |  |
**next_token** | Option<**String**> | Indicates where to start paging for queries that support pagination. |  |
**diffs** | Option<**bool**> | Boolean flag indicating whether to compute job diffs. |  |

### Return type

[**crate::models::JobVersionsResponse**](JobVersionsResponse.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_jobs

> Vec<crate::models::JobListStub> get_jobs(region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token)


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

[**Vec<crate::models::JobListStub>**](JobListStub.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_job

> crate::models::JobRegisterResponse post_job(job_name, job_register_request, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
**job_register_request** | [**JobRegisterRequest**](JobRegisterRequest.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

[**crate::models::JobRegisterResponse**](JobRegisterResponse.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_job_dispatch

> crate::models::JobDispatchResponse post_job_dispatch(job_name, job_dispatch_request, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
**job_dispatch_request** | [**JobDispatchRequest**](JobDispatchRequest.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

[**crate::models::JobDispatchResponse**](JobDispatchResponse.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_job_evaluate

> crate::models::JobRegisterResponse post_job_evaluate(job_name, job_evaluate_request, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
**job_evaluate_request** | [**JobEvaluateRequest**](JobEvaluateRequest.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

[**crate::models::JobRegisterResponse**](JobRegisterResponse.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_job_parse

> crate::models::Job post_job_parse(jobs_parse_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**jobs_parse_request** | [**JobsParseRequest**](JobsParseRequest.md) |  | [required] |

### Return type

[**crate::models::Job**](Job.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_job_periodic_force

> crate::models::PeriodicForceResponse post_job_periodic_force(job_name, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

[**crate::models::PeriodicForceResponse**](PeriodicForceResponse.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_job_plan

> crate::models::JobPlanResponse post_job_plan(job_name, job_plan_request, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
**job_plan_request** | [**JobPlanRequest**](JobPlanRequest.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

[**crate::models::JobPlanResponse**](JobPlanResponse.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_job_revert

> crate::models::JobRegisterResponse post_job_revert(job_name, job_revert_request, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
**job_revert_request** | [**JobRevertRequest**](JobRevertRequest.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

[**crate::models::JobRegisterResponse**](JobRegisterResponse.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_job_scaling_request

> crate::models::JobRegisterResponse post_job_scaling_request(job_name, scaling_request, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
**scaling_request** | [**ScalingRequest**](ScalingRequest.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

[**crate::models::JobRegisterResponse**](JobRegisterResponse.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_job_stability

> crate::models::JobStabilityResponse post_job_stability(job_name, job_stability_request, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_name** | **String** | The job identifier. | [required] |
**job_stability_request** | [**JobStabilityRequest**](JobStabilityRequest.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

[**crate::models::JobStabilityResponse**](JobStabilityResponse.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_job_validate_request

> crate::models::JobValidateResponse post_job_validate_request(job_validate_request, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_validate_request** | [**JobValidateRequest**](JobValidateRequest.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

[**crate::models::JobValidateResponse**](JobValidateResponse.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## register_job

> crate::models::JobRegisterResponse register_job(job_register_request, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_register_request** | [**JobRegisterRequest**](JobRegisterRequest.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

[**crate::models::JobRegisterResponse**](JobRegisterResponse.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

