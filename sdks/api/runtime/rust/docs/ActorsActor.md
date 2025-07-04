# ActorsActor

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | Can be a UUID or base36 encoded binary data. | 
**region** | **String** |  | 
**tags** | Option<[**serde_json::Value**](.md)> |  | 
**build** | [**uuid::Uuid**](uuid::Uuid.md) |  | 
**ports** | [**::std::collections::HashMap<String, crate::models::ActorsPort>**](ActorsPort.md) |  | 
**kill_timeout** | Option<**i64**> | The duration to wait for in milliseconds before force killing the actor after a DELETE request. This gives the actor time to perform a shutdown sequence before being killed. This should be set to a safe default, and can be overridden during a DELETE request if needed. | [optional]
**durable** | Option<**bool**> | If true, the actor will try to reschedule itself automatically in the event of a crash or a datacenter failover. The actor will not reschedule if it exits successfully. | [optional]
**created_at** | **String** | RFC3339 timestamp | 
**started_at** | Option<**String**> | RFC3339 timestamp | [optional]
**destroyed_at** | Option<**String**> | RFC3339 timestamp | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


