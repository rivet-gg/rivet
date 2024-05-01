# \DynamicServersServersApi

All URIs are relative to *https://api.rivet.gg*

Method | HTTP request | Description
------------- | ------------- | -------------
[**dynamic_servers_servers_create**](DynamicServersServersApi.md#dynamic_servers_servers_create) | **POST** /dynamic-servers/servers | 
[**dynamic_servers_servers_destroy**](DynamicServersServersApi.md#dynamic_servers_servers_destroy) | **DELETE** /dynamic-servers/servers/{server_id} | 



## dynamic_servers_servers_create

> crate::models::DynamicServersCreateServerResponse dynamic_servers_servers_create(dynamic_servers_create_server_request)


Create a new dynamic server.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dynamic_servers_create_server_request** | [**DynamicServersCreateServerRequest**](DynamicServersCreateServerRequest.md) |  | [required] |

### Return type

[**crate::models::DynamicServersCreateServerResponse**](DynamicServersCreateServerResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## dynamic_servers_servers_destroy

> crate::models::DynamicServersDestroyServerResponse dynamic_servers_servers_destroy(server_id, override_kill_timeout)


Destroy a dynamic server.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**server_id** | **uuid::Uuid** | The id of the server to destroy | [required] |
**override_kill_timeout** | Option<**i64**> | The duration to wait for in milliseconds before killing the server. This should be used to override the default kill timeout if a faster time is needed, say for ignoring a graceful shutdown. |  |

### Return type

[**crate::models::DynamicServersDestroyServerResponse**](DynamicServersDestroyServerResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

