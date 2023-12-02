# \ACLApi

All URIs are relative to *http://127.0.0.1:4646/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_acl_policy**](ACLApi.md#delete_acl_policy) | **DELETE** /acl/policy/{policyName} | 
[**delete_acl_token**](ACLApi.md#delete_acl_token) | **DELETE** /acl/token/{tokenAccessor} | 
[**get_acl_policies**](ACLApi.md#get_acl_policies) | **GET** /acl/policies | 
[**get_acl_policy**](ACLApi.md#get_acl_policy) | **GET** /acl/policy/{policyName} | 
[**get_acl_token**](ACLApi.md#get_acl_token) | **GET** /acl/token/{tokenAccessor} | 
[**get_acl_token_self**](ACLApi.md#get_acl_token_self) | **GET** /acl/token | 
[**get_acl_tokens**](ACLApi.md#get_acl_tokens) | **GET** /acl/tokens | 
[**post_acl_bootstrap**](ACLApi.md#post_acl_bootstrap) | **POST** /acl/bootstrap | 
[**post_acl_policy**](ACLApi.md#post_acl_policy) | **POST** /acl/policy/{policyName} | 
[**post_acl_token**](ACLApi.md#post_acl_token) | **POST** /acl/token/{tokenAccessor} | 
[**post_acl_token_onetime**](ACLApi.md#post_acl_token_onetime) | **POST** /acl/token/onetime | 
[**post_acl_token_onetime_exchange**](ACLApi.md#post_acl_token_onetime_exchange) | **POST** /acl/token/onetime/exchange | 



## delete_acl_policy

> delete_acl_policy(policy_name, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**policy_name** | **String** | The ACL policy name. | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

 (empty response body)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_acl_token

> delete_acl_token(token_accessor, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**token_accessor** | **String** | The token accessor ID. | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

 (empty response body)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_acl_policies

> Vec<crate::models::AclPolicyListStub> get_acl_policies(region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token)


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

[**Vec<crate::models::AclPolicyListStub>**](ACLPolicyListStub.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_acl_policy

> crate::models::AclPolicy get_acl_policy(policy_name, region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**policy_name** | **String** | The ACL policy name. | [required] |
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

[**crate::models::AclPolicy**](ACLPolicy.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_acl_token

> crate::models::AclToken get_acl_token(token_accessor, region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**token_accessor** | **String** | The token accessor ID. | [required] |
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

[**crate::models::AclToken**](ACLToken.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_acl_token_self

> crate::models::AclToken get_acl_token_self(region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token)


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

[**crate::models::AclToken**](ACLToken.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_acl_tokens

> Vec<crate::models::AclTokenListStub> get_acl_tokens(region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token)


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

[**Vec<crate::models::AclTokenListStub>**](ACLTokenListStub.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_acl_bootstrap

> crate::models::AclToken post_acl_bootstrap(region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

[**crate::models::AclToken**](ACLToken.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_acl_policy

> post_acl_policy(policy_name, acl_policy, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**policy_name** | **String** | The ACL policy name. | [required] |
**acl_policy** | [**AclPolicy**](AclPolicy.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

 (empty response body)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_acl_token

> crate::models::AclToken post_acl_token(token_accessor, acl_token, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**token_accessor** | **String** | The token accessor ID. | [required] |
**acl_token** | [**AclToken**](AclToken.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

[**crate::models::AclToken**](ACLToken.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_acl_token_onetime

> crate::models::OneTimeToken post_acl_token_onetime(region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

[**crate::models::OneTimeToken**](OneTimeToken.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_acl_token_onetime_exchange

> crate::models::AclToken post_acl_token_onetime_exchange(one_time_token_exchange_request, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**one_time_token_exchange_request** | [**OneTimeTokenExchangeRequest**](OneTimeTokenExchangeRequest.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

[**crate::models::AclToken**](ACLToken.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

