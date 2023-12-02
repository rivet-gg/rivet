# Allocation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**alloc_modify_index** | Option<**i32**> |  | [optional]
**allocated_resources** | Option<[**crate::models::AllocatedResources**](AllocatedResources.md)> |  | [optional]
**client_description** | Option<**String**> |  | [optional]
**client_status** | Option<**String**> |  | [optional]
**create_index** | Option<**i32**> |  | [optional]
**create_time** | Option<**i64**> |  | [optional]
**deployment_id** | Option<**String**> |  | [optional]
**deployment_status** | Option<[**crate::models::AllocDeploymentStatus**](AllocDeploymentStatus.md)> |  | [optional]
**desired_description** | Option<**String**> |  | [optional]
**desired_status** | Option<**String**> |  | [optional]
**desired_transition** | Option<[**crate::models::DesiredTransition**](DesiredTransition.md)> |  | [optional]
**eval_id** | Option<**String**> |  | [optional]
**followup_eval_id** | Option<**String**> |  | [optional]
**ID** | Option<**String**> |  | [optional]
**job** | Option<[**crate::models::Job**](Job.md)> |  | [optional]
**job_id** | Option<**String**> |  | [optional]
**metrics** | Option<[**crate::models::AllocationMetric**](AllocationMetric.md)> |  | [optional]
**modify_index** | Option<**i32**> |  | [optional]
**modify_time** | Option<**i64**> |  | [optional]
**name** | Option<**String**> |  | [optional]
**namespace** | Option<**String**> |  | [optional]
**next_allocation** | Option<**String**> |  | [optional]
**node_id** | Option<**String**> |  | [optional]
**node_name** | Option<**String**> |  | [optional]
**preempted_allocations** | Option<**Vec<String>**> |  | [optional]
**preempted_by_allocation** | Option<**String**> |  | [optional]
**previous_allocation** | Option<**String**> |  | [optional]
**reschedule_tracker** | Option<[**crate::models::RescheduleTracker**](RescheduleTracker.md)> |  | [optional]
**resources** | Option<[**crate::models::Resources**](Resources.md)> |  | [optional]
**services** | Option<**::std::collections::HashMap<String, String>**> |  | [optional]
**task_group** | Option<**String**> |  | [optional]
**task_resources** | Option<[**::std::collections::HashMap<String, crate::models::Resources>**](Resources.md)> |  | [optional]
**task_states** | Option<[**::std::collections::HashMap<String, crate::models::TaskState>**](TaskState.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


