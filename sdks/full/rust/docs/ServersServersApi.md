# \ServersServersApi

All URIs are relative to *https://api.rivet.gg*

Method | HTTP request | Description
------------- | ------------- | -------------
[**servers_servers_create**](ServersServersApi.md#servers_servers_create) | **POST** /servers/servers | 
[**servers_servers_destroy**](ServersServersApi.md#servers_servers_destroy) | **DELETE** /servers/servers/{server_id} | 



## servers_servers_create

> crate::models::ServersServersCreateResponse servers_servers_create(servers_servers_create_request)


Create a new dynamic server.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**servers_servers_create_request** | [**ServersServersCreateRequest**](ServersServersCreateRequest.md) |  | [required] |

### Return type

[**crate::models::ServersServersCreateResponse**](ServersServersCreateResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## servers_servers_destroy

> servers_servers_destroy(server_id, servers_servers_destroy_request)


Destroy a dynamic server.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**server_id** | **uuid::Uuid** | The id of the server to destroy | [required] |
**servers_servers_destroy_request** | [**ServersServersDestroyRequest**](ServersServersDestroyRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

