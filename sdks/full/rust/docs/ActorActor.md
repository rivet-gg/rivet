# ActorActor

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **i64** |  | 
**datacenter** | [**uuid::Uuid**](uuid::Uuid.md) |  | 
**destroyed_at** | Option<**i64**> |  | [optional]
**environment** | [**uuid::Uuid**](uuid::Uuid.md) |  | 
**id** | [**uuid::Uuid**](uuid::Uuid.md) |  | 
**lifecycle** | [**crate::models::ActorLifecycle**](ActorLifecycle.md) |  | 
**network** | [**crate::models::ActorNetwork**](ActorNetwork.md) |  | 
**resources** | [**crate::models::ActorResources**](ActorResources.md) |  | 
**runtime** | [**crate::models::ActorRuntime**](ActorRuntime.md) |  | 
**started_at** | Option<**i64**> |  | [optional]
**tags** | Option<[**serde_json::Value**](.md)> |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


