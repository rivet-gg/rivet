# \CloudGamesTokensApi

All URIs are relative to *https://api.rivet.gg*

Method | HTTP request | Description
------------- | ------------- | -------------
[**cloud_games_tokens_create_cloud_token**](CloudGamesTokensApi.md#cloud_games_tokens_create_cloud_token) | **POST** /cloud/games/{game_id}/tokens/cloud | 
[**cloud_games_tokens_create_service_token**](CloudGamesTokensApi.md#cloud_games_tokens_create_service_token) | **POST** /cloud/games/{game_id}/tokens/service | 



## cloud_games_tokens_create_cloud_token

> crate::models::CloudGamesCreateCloudTokenResponse cloud_games_tokens_create_cloud_token(game_id)


Creates a new game cloud token.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**game_id** | **uuid::Uuid** |  | [required] |

### Return type

[**crate::models::CloudGamesCreateCloudTokenResponse**](CloudGamesCreateCloudTokenResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## cloud_games_tokens_create_service_token

> crate::models::CloudGamesCreateServiceTokenResponse cloud_games_tokens_create_service_token(game_id, cloud_games_create_service_token_request)


Creates a new service token.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**game_id** | **uuid::Uuid** |  | [required] |
**cloud_games_create_service_token_request** | [**CloudGamesCreateServiceTokenRequest**](CloudGamesCreateServiceTokenRequest.md) |  | [required] |

### Return type

[**crate::models::CloudGamesCreateServiceTokenResponse**](CloudGamesCreateServiceTokenResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

