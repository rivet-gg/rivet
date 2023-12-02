# CsiVolume

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**access_mode** | Option<**String**> |  | [optional]
**allocations** | Option<[**Vec<crate::models::AllocationListStub>**](AllocationListStub.md)> |  | [optional]
**attachment_mode** | Option<**String**> |  | [optional]
**capacity** | Option<**i64**> |  | [optional]
**clone_id** | Option<**String**> |  | [optional]
**context** | Option<**::std::collections::HashMap<String, String>**> |  | [optional]
**controller_required** | Option<**bool**> |  | [optional]
**controllers_expected** | Option<**i32**> |  | [optional]
**controllers_healthy** | Option<**i32**> |  | [optional]
**create_index** | Option<**i32**> |  | [optional]
**external_id** | Option<**String**> |  | [optional]
**ID** | Option<**String**> |  | [optional]
**modify_index** | Option<**i32**> |  | [optional]
**mount_options** | Option<[**crate::models::CsiMountOptions**](CSIMountOptions.md)> |  | [optional]
**name** | Option<**String**> |  | [optional]
**namespace** | Option<**String**> |  | [optional]
**nodes_expected** | Option<**i32**> |  | [optional]
**nodes_healthy** | Option<**i32**> |  | [optional]
**parameters** | Option<**::std::collections::HashMap<String, String>**> |  | [optional]
**plugin_id** | Option<**String**> |  | [optional]
**provider** | Option<**String**> |  | [optional]
**provider_version** | Option<**String**> |  | [optional]
**read_allocs** | Option<[**::std::collections::HashMap<String, crate::models::Allocation>**](Allocation.md)> |  | [optional]
**requested_capabilities** | Option<[**Vec<crate::models::CsiVolumeCapability>**](CSIVolumeCapability.md)> |  | [optional]
**requested_capacity_max** | Option<**i64**> |  | [optional]
**requested_capacity_min** | Option<**i64**> |  | [optional]
**requested_topologies** | Option<[**crate::models::CsiTopologyRequest**](CSITopologyRequest.md)> |  | [optional]
**resource_exhausted** | Option<**String**> |  | [optional]
**schedulable** | Option<**bool**> |  | [optional]
**secrets** | Option<**::std::collections::HashMap<String, String>**> |  | [optional]
**snapshot_id** | Option<**String**> |  | [optional]
**topologies** | Option<[**Vec<crate::models::CsiTopology>**](CSITopology.md)> |  | [optional]
**write_allocs** | Option<[**::std::collections::HashMap<String, crate::models::Allocation>**](Allocation.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


