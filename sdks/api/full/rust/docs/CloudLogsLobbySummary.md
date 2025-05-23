# CloudLogsLobbySummary

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**lobby_id** | [**uuid::Uuid**](uuid::Uuid.md) |  | 
**namespace_id** | [**uuid::Uuid**](uuid::Uuid.md) |  | 
**lobby_group_name_id** | **String** | A human readable short identifier used to references resources. Different than a `rivet.common#Uuid` because this is intended to be human readable. Different than `rivet.common#DisplayName` because this should not include special characters and be short. | 
**region_id** | [**uuid::Uuid**](uuid::Uuid.md) |  | 
**create_ts** | **String** | RFC3339 timestamp | 
**start_ts** | Option<**String**> | RFC3339 timestamp | [optional]
**ready_ts** | Option<**String**> | RFC3339 timestamp | [optional]
**status** | [**crate::models::CloudLogsLobbyStatus**](CloudLogsLobbyStatus.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


