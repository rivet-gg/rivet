# BuildsBuild

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | [**uuid::Uuid**](uuid::Uuid.md) |  | 
**name** | **String** |  | 
**created_at** | **String** | RFC3339 timestamp | 
**content_length** | **i64** | Unsigned 64 bit integer. | 
**allocation** | Option<[**crate::models::BuildsAllocation**](BuildsAllocation.md)> |  | [optional]
**resources** | Option<[**crate::models::BuildsResources**](BuildsResources.md)> |  | [optional]
**tags** | **::std::collections::HashMap<String, String>** | Tags of this build | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


