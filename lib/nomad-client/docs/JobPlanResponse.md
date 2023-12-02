# JobPlanResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**annotations** | Option<[**crate::models::PlanAnnotations**](PlanAnnotations.md)> |  | [optional]
**created_evals** | Option<[**Vec<crate::models::Evaluation>**](Evaluation.md)> |  | [optional]
**diff** | Option<[**crate::models::JobDiff**](JobDiff.md)> |  | [optional]
**failed_tg_allocs** | Option<[**::std::collections::HashMap<String, crate::models::AllocationMetric>**](AllocationMetric.md)> |  | [optional]
**job_modify_index** | Option<**i32**> |  | [optional]
**next_periodic_launch** | Option<**String**> |  | [optional]
**warnings** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


