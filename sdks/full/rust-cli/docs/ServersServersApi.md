# \ServersServersApi

All URIs are relative to *https://api.rivet.gg*

Method | HTTP request | Description
------------- | ------------- | -------------
[**servers_servers_create**](ServersServersApi.md#servers_servers_create) | **POST** /servers/servers | 
[**servers_servers_destroy**](ServersServersApi.md#servers_servers_destroy) | **DELETE** /servers/servers/{server_id} | 



## servers_servers_create

> crate::models::ServersCreateServerResponse servers_servers_create(servers_create_server_request)


Create a new dynamic server.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**servers_create_server_request** | [**ServersCreateServerRequest**](ServersCreateServerRequest.md) |  | [required] |

### Return type

[**crate::models::ServersCreateServerResponse**](ServersCreateServerResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## servers_servers_destroy

> crate::models::ServersDestroyServerResponse servers_servers_destroy(server_id, override_kill_timeout)


Destroy a dynamic server.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**server_id** | **uuid::Uuid** | The id of the server to destroy | [required] |
**override_kill_timeout** | Option<**i64**> | The duration to wait for in milliseconds before killing the server. This should be used to override the default kill timeout if a faster time is needed, say for ignoring a graceful shutdown. |  |

### Return type

[**crate::models::ServersDestroyServerResponse**](ServersDestroyServerResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

