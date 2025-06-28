# \ActorsV1Api

All URIs are relative to *https://api.rivet.gg*

Method | HTTP request | Description
------------- | ------------- | -------------
[**actors_v1_create**](ActorsV1Api.md#actors_v1_create) | **POST** /actors | 
[**actors_v1_destroy**](ActorsV1Api.md#actors_v1_destroy) | **DELETE** /actors/{actor} | 
[**actors_v1_get**](ActorsV1Api.md#actors_v1_get) | **GET** /actors/{actor} | 
[**actors_v1_list**](ActorsV1Api.md#actors_v1_list) | **GET** /actors | 
[**actors_v1_upgrade**](ActorsV1Api.md#actors_v1_upgrade) | **POST** /actors/{actor}/upgrade | 
[**actors_v1_upgrade_all**](ActorsV1Api.md#actors_v1_upgrade_all) | **POST** /actors/upgrade | 



## actors_v1_create

> crate::models::ActorsV1CreateActorResponse actors_v1_create(actors_v1_create_actor_request, project, environment, endpoint_type)


Create a new actor.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actors_v1_create_actor_request** | [**ActorsV1CreateActorRequest**](ActorsV1CreateActorRequest.md) |  | [required] |
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |
**endpoint_type** | Option<[**ActorsV1EndpointType**](.md)> |  |  |

### Return type

[**crate::models::ActorsV1CreateActorResponse**](ActorsV1CreateActorResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actors_v1_destroy

> serde_json::Value actors_v1_destroy(actor, project, environment, override_kill_timeout)


Destroy a actor.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor** | **uuid::Uuid** | The id of the actor to destroy | [required] |
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |
**override_kill_timeout** | Option<**i64**> | The duration to wait for in milliseconds before killing the actor. This should be used to override the default kill timeout if a faster time is needed, say for ignoring a graceful shutdown. |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actors_v1_get

> crate::models::ActorsV1GetActorResponse actors_v1_get(actor, project, environment, endpoint_type)


Gets a actor.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor** | **uuid::Uuid** | The id of the actor to destroy | [required] |
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |
**endpoint_type** | Option<[**ActorsV1EndpointType**](.md)> |  |  |

### Return type

[**crate::models::ActorsV1GetActorResponse**](ActorsV1GetActorResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actors_v1_list

> crate::models::ActorsV1ListActorsResponse actors_v1_list(project, environment, endpoint_type, tags_json, include_destroyed, cursor)


Lists all actors associated with the token used. Can be filtered by tags in the query string.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |
**endpoint_type** | Option<[**ActorsV1EndpointType**](.md)> |  |  |
**tags_json** | Option<**String**> |  |  |
**include_destroyed** | Option<**bool**> |  |  |
**cursor** | Option<**String**> |  |  |

### Return type

[**crate::models::ActorsV1ListActorsResponse**](ActorsV1ListActorsResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actors_v1_upgrade

> serde_json::Value actors_v1_upgrade(actor, actors_v1_upgrade_actor_request, project, environment)


Upgrades a actor.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor** | **uuid::Uuid** | The id of the actor to upgrade | [required] |
**actors_v1_upgrade_actor_request** | [**ActorsV1UpgradeActorRequest**](ActorsV1UpgradeActorRequest.md) |  | [required] |
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


## actors_v1_upgrade_all

> crate::models::ActorsV1UpgradeAllActorsResponse actors_v1_upgrade_all(actors_v1_upgrade_all_actors_request, project, environment)


Upgrades all actors matching the given tags.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actors_v1_upgrade_all_actors_request** | [**ActorsV1UpgradeAllActorsRequest**](ActorsV1UpgradeAllActorsRequest.md) |  | [required] |
**project** | Option<**String**> |  |  |
**environment** | Option<**String**> |  |  |

### Return type

[**crate::models::ActorsV1UpgradeAllActorsResponse**](ActorsV1UpgradeAllActorsResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

