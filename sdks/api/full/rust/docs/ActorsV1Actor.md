# ActorsV1Actor

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | [**uuid::Uuid**](uuid::Uuid.md) |  | 
**region** | **String** |  | 
**tags** | Option<[**serde_json::Value**](.md)> |  | 
**runtime** | [**crate::models::ActorsV1Runtime**](ActorsV1Runtime.md) |  | 
**network** | [**crate::models::ActorsV1Network**](ActorsV1Network.md) |  | 
**resources** | Option<[**crate::models::ActorsV1Resources**](ActorsV1Resources.md)> |  | [optional]
**lifecycle** | [**crate::models::ActorsV1Lifecycle**](ActorsV1Lifecycle.md) |  | 
**created_at** | **String** | RFC3339 timestamp | 
**started_at** | Option<**String**> | RFC3339 timestamp | [optional]
**destroyed_at** | Option<**String**> | RFC3339 timestamp | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


