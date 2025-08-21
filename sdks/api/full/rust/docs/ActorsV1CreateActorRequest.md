# ActorsV1CreateActorRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**region** | Option<**String**> |  | [optional]
**tags** | Option<[**serde_json::Value**](.md)> |  | 
**build** | Option<[**uuid::Uuid**](uuid::Uuid.md)> |  | [optional]
**build_tags** | Option<[**serde_json::Value**](.md)> |  | [optional]
**runtime** | Option<[**crate::models::ActorsV1CreateActorRuntimeRequest**](ActorsV1CreateActorRuntimeRequest.md)> |  | [optional]
**network** | Option<[**crate::models::ActorsV1CreateActorNetworkRequest**](ActorsV1CreateActorNetworkRequest.md)> |  | [optional]
**resources** | Option<[**crate::models::ActorsV1Resources**](ActorsV1Resources.md)> |  | [optional]
**lifecycle** | Option<[**crate::models::ActorsV1Lifecycle**](ActorsV1Lifecycle.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


