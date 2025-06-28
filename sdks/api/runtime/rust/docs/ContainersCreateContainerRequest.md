# ContainersCreateContainerRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**region** | Option<**String**> |  | [optional]
**tags** | Option<[**serde_json::Value**](.md)> |  | 
**build** | Option<[**uuid::Uuid**](uuid::Uuid.md)> |  | [optional]
**build_tags** | Option<[**serde_json::Value**](.md)> |  | [optional]
**runtime** | Option<[**crate::models::ContainersCreateContainerRuntimeRequest**](ContainersCreateContainerRuntimeRequest.md)> |  | [optional]
**network** | Option<[**crate::models::ContainersCreateContainerNetworkRequest**](ContainersCreateContainerNetworkRequest.md)> |  | [optional]
**resources** | [**crate::models::ContainersResources**](ContainersResources.md) |  | 
**lifecycle** | Option<[**crate::models::ContainersLifecycle**](ContainersLifecycle.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


