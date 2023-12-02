# Task

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**affinities** | Option<[**Vec<crate::models::Affinity>**](Affinity.md)> |  | [optional]
**artifacts** | Option<[**Vec<crate::models::TaskArtifact>**](TaskArtifact.md)> |  | [optional]
**csi_plugin_config** | Option<[**crate::models::TaskCsiPluginConfig**](TaskCSIPluginConfig.md)> |  | [optional]
**config** | Option<[**::std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md)> |  | [optional]
**constraints** | Option<[**Vec<crate::models::Constraint>**](Constraint.md)> |  | [optional]
**dispatch_payload** | Option<[**crate::models::DispatchPayloadConfig**](DispatchPayloadConfig.md)> |  | [optional]
**driver** | Option<**String**> |  | [optional]
**env** | Option<**::std::collections::HashMap<String, String>**> |  | [optional]
**kill_signal** | Option<**String**> |  | [optional]
**kill_timeout** | Option<**i64**> |  | [optional]
**kind** | Option<**String**> |  | [optional]
**leader** | Option<**bool**> |  | [optional]
**lifecycle** | Option<[**crate::models::TaskLifecycle**](TaskLifecycle.md)> |  | [optional]
**log_config** | Option<[**crate::models::LogConfig**](LogConfig.md)> |  | [optional]
**meta** | Option<**::std::collections::HashMap<String, String>**> |  | [optional]
**name** | Option<**String**> |  | [optional]
**resources** | Option<[**crate::models::Resources**](Resources.md)> |  | [optional]
**restart_policy** | Option<[**crate::models::RestartPolicy**](RestartPolicy.md)> |  | [optional]
**scaling_policies** | Option<[**Vec<crate::models::ScalingPolicy>**](ScalingPolicy.md)> |  | [optional]
**services** | Option<[**Vec<crate::models::Service>**](Service.md)> |  | [optional]
**shutdown_delay** | Option<**i64**> |  | [optional]
**templates** | Option<[**Vec<crate::models::Template>**](Template.md)> |  | [optional]
**user** | Option<**String**> |  | [optional]
**vault** | Option<[**crate::models::Vault**](Vault.md)> |  | [optional]
**volume_mounts** | Option<[**Vec<crate::models::VolumeMount>**](VolumeMount.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


