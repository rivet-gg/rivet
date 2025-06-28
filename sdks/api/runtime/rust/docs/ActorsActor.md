# ActorsActor

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | Can be a UUID or base36 encoded binary data. | 
**region** | **String** |  | 
**tags** | Option<[**serde_json::Value**](.md)> |  | 
**runtime** | [**crate::models::ActorsRuntime**](ActorsRuntime.md) |  | 
**network** | [**crate::models::ActorsNetwork**](ActorsNetwork.md) |  | 
**lifecycle** | [**crate::models::ActorsLifecycle**](ActorsLifecycle.md) |  | 
**created_at** | **String** | RFC3339 timestamp | 
**started_at** | Option<**String**> | RFC3339 timestamp | [optional]
**destroyed_at** | Option<**String**> | RFC3339 timestamp | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


