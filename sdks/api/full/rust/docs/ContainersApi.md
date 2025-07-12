# \ContainersApi

All URIs are relative to *https://api.rivet.gg*

Method | HTTP request | Description
------------- | ------------- | -------------
[**containers_create**](ContainersApi.md#containers_create) | **POST** /v1/containers | 
[**containers_destroy**](ContainersApi.md#containers_destroy) | **DELETE** /v1/containers/{container} | 
[**containers_get**](ContainersApi.md#containers_get) | **GET** /v1/containers/{container} | 
[**containers_list**](ContainersApi.md#containers_list) | **GET** /v1/containers | 
[**containers_upgrade**](ContainersApi.md#containers_upgrade) | **POST** /v1/containers/{container}/upgrade | 
[**containers_upgrade_all**](ContainersApi.md#containers_upgrade_all) | **POST** /v1/containers/upgrade | 



## containers_create

> crate::models::ContainersCreateContainerResponse containers_create(containers_create_container_request, project, environment, endpoint_type)


Create a new container.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**containers_create_container_request** | [**ContainersCreateContainerRequest**](ContainersCreateContainerRequest.md) |  | [required] |
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |
**endpoint_type** | Option<[**ContainersEndpointType**](.md)> |  |  |

### Return type

[**crate::models::ContainersCreateContainerResponse**](ContainersCreateContainerResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## containers_destroy

> serde_json::Value containers_destroy(container, project, environment, override_kill_timeout)


Destroy a container.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**container** | **String** | The id of the container to destroy | [required] |
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |
**override_kill_timeout** | Option<**i64**> | The duration to wait for in milliseconds before killing the container. This should be used to override the default kill timeout if a faster time is needed, say for ignoring a graceful shutdown. |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## containers_get

> crate::models::ContainersGetContainerResponse containers_get(container, project, environment, endpoint_type)


Gets a container.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**container** | **String** | The id of the container to destroy | [required] |
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |
**endpoint_type** | Option<[**ContainersEndpointType**](.md)> |  |  |

### Return type

[**crate::models::ContainersGetContainerResponse**](ContainersGetContainerResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## containers_list

> crate::models::ContainersListContainersResponse containers_list(project, environment, endpoint_type, tags_json, include_destroyed, cursor)


Lists all containers associated with the token used. Can be filtered by tags in the query string.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |
**endpoint_type** | Option<[**ContainersEndpointType**](.md)> |  |  |
**tags_json** | Option<**String**> |  |  |
**include_destroyed** | Option<**bool**> |  |  |
**cursor** | Option<**String**> |  |  |

### Return type

[**crate::models::ContainersListContainersResponse**](ContainersListContainersResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## containers_upgrade

> serde_json::Value containers_upgrade(container, containers_upgrade_container_request, project, environment)


Upgrades a container.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**container** | **String** | The id of the container to upgrade | [required] |
**containers_upgrade_container_request** | [**ContainersUpgradeContainerRequest**](ContainersUpgradeContainerRequest.md) |  | [required] |
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## containers_upgrade_all

> crate::models::ContainersUpgradeAllContainersResponse containers_upgrade_all(containers_upgrade_all_containers_request, project, environment)


Upgrades all containers matching the given tags.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**containers_upgrade_all_containers_request** | [**ContainersUpgradeAllContainersRequest**](ContainersUpgradeAllContainersRequest.md) |  | [required] |
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |

### Return type

[**crate::models::ContainersUpgradeAllContainersResponse**](ContainersUpgradeAllContainersResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

