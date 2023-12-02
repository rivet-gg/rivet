# Job

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**affinities** | Option<[**Vec<crate::models::Affinity>**](Affinity.md)> |  | [optional]
**all_at_once** | Option<**bool**> |  | [optional]
**constraints** | Option<[**Vec<crate::models::Constraint>**](Constraint.md)> |  | [optional]
**consul_namespace** | Option<**String**> |  | [optional]
**consul_token** | Option<**String**> |  | [optional]
**create_index** | Option<**i32**> |  | [optional]
**datacenters** | Option<**Vec<String>**> |  | [optional]
**dispatch_idempotency_token** | Option<**String**> |  | [optional]
**dispatched** | Option<**bool**> |  | [optional]
**ID** | Option<**String**> |  | [optional]
**job_modify_index** | Option<**i32**> |  | [optional]
**meta** | Option<**::std::collections::HashMap<String, String>**> |  | [optional]
**migrate** | Option<[**crate::models::MigrateStrategy**](MigrateStrategy.md)> |  | [optional]
**modify_index** | Option<**i32**> |  | [optional]
**multiregion** | Option<[**crate::models::Multiregion**](Multiregion.md)> |  | [optional]
**name** | Option<**String**> |  | [optional]
**namespace** | Option<**String**> |  | [optional]
**nomad_token_id** | Option<**String**> |  | [optional]
**parameterized_job** | Option<[**crate::models::ParameterizedJobConfig**](ParameterizedJobConfig.md)> |  | [optional]
**parent_id** | Option<**String**> |  | [optional]
**payload** | Option<**String**> |  | [optional]
**periodic** | Option<[**crate::models::PeriodicConfig**](PeriodicConfig.md)> |  | [optional]
**priority** | Option<**i32**> |  | [optional]
**region** | Option<**String**> |  | [optional]
**reschedule** | Option<[**crate::models::ReschedulePolicy**](ReschedulePolicy.md)> |  | [optional]
**spreads** | Option<[**Vec<crate::models::Spread>**](Spread.md)> |  | [optional]
**stable** | Option<**bool**> |  | [optional]
**status** | Option<**String**> |  | [optional]
**status_description** | Option<**String**> |  | [optional]
**stop** | Option<**bool**> |  | [optional]
**submit_time** | Option<**i64**> |  | [optional]
**task_groups** | Option<[**Vec<crate::models::TaskGroup>**](TaskGroup.md)> |  | [optional]
**_type** | Option<**String**> |  | [optional]
**update** | Option<[**crate::models::UpdateStrategy**](UpdateStrategy.md)> |  | [optional]
**vault_namespace** | Option<**String**> |  | [optional]
**vault_token** | Option<**String**> |  | [optional]
**version** | Option<**i32**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


