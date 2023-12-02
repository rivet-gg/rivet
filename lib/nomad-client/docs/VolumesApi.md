# \VolumesApi

All URIs are relative to *http://127.0.0.1:4646/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_volume**](VolumesApi.md#create_volume) | **POST** /volume/csi/{volumeId}/{action} | 
[**delete_snapshot**](VolumesApi.md#delete_snapshot) | **DELETE** /volumes/snapshot | 
[**delete_volume_registration**](VolumesApi.md#delete_volume_registration) | **DELETE** /volume/csi/{volumeId} | 
[**detach_or_delete_volume**](VolumesApi.md#detach_or_delete_volume) | **DELETE** /volume/csi/{volumeId}/{action} | 
[**get_external_volumes**](VolumesApi.md#get_external_volumes) | **GET** /volumes/external | 
[**get_snapshots**](VolumesApi.md#get_snapshots) | **GET** /volumes/snapshot | 
[**get_volume**](VolumesApi.md#get_volume) | **GET** /volume/csi/{volumeId} | 
[**get_volumes**](VolumesApi.md#get_volumes) | **GET** /volumes | 
[**post_snapshot**](VolumesApi.md#post_snapshot) | **POST** /volumes/snapshot | 
[**post_volume**](VolumesApi.md#post_volume) | **POST** /volumes | 
[**post_volume_registration**](VolumesApi.md#post_volume_registration) | **POST** /volume/csi/{volumeId} | 



## create_volume

> create_volume(volume_id, action, csi_volume_create_request, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**volume_id** | **String** | Volume unique identifier. | [required] |
**action** | **String** | The action to perform on the Volume (create, detach, delete). | [required] |
**csi_volume_create_request** | [**CsiVolumeCreateRequest**](CsiVolumeCreateRequest.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

 (empty response body)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_snapshot

> delete_snapshot(region, namespace, x_nomad_token, idempotency_token, plugin_id, snapshot_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |
**plugin_id** | Option<**String**> | Filters volume lists by plugin ID. |  |
**snapshot_id** | Option<**String**> | The ID of the snapshot to target. |  |

### Return type

 (empty response body)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_volume_registration

> delete_volume_registration(volume_id, region, namespace, x_nomad_token, idempotency_token, force)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**volume_id** | **String** | Volume unique identifier. | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |
**force** | Option<**String**> | Used to force the de-registration of a volume. |  |

### Return type

 (empty response body)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## detach_or_delete_volume

> detach_or_delete_volume(volume_id, action, region, namespace, x_nomad_token, idempotency_token, node)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**volume_id** | **String** | Volume unique identifier. | [required] |
**action** | **String** | The action to perform on the Volume (create, detach, delete). | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |
**node** | Option<**String**> | Specifies node to target volume operation for. |  |

### Return type

 (empty response body)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_external_volumes

> crate::models::CsiVolumeListExternalResponse get_external_volumes(region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token, plugin_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**index** | Option<**i32**> | If set, wait until query exceeds given index. Must be provided with WaitParam. |  |
**wait** | Option<**String**> | Provided with IndexParam to wait for change. |  |
**stale** | Option<**String**> | If present, results will include stale reads. |  |
**prefix** | Option<**String**> | Constrains results to jobs that start with the defined prefix |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**per_page** | Option<**i32**> | Maximum number of results to return. |  |
**next_token** | Option<**String**> | Indicates where to start paging for queries that support pagination. |  |
**plugin_id** | Option<**String**> | Filters volume lists by plugin ID. |  |

### Return type

[**crate::models::CsiVolumeListExternalResponse**](CSIVolumeListExternalResponse.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_snapshots

> crate::models::CsiSnapshotListResponse get_snapshots(region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token, plugin_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**index** | Option<**i32**> | If set, wait until query exceeds given index. Must be provided with WaitParam. |  |
**wait** | Option<**String**> | Provided with IndexParam to wait for change. |  |
**stale** | Option<**String**> | If present, results will include stale reads. |  |
**prefix** | Option<**String**> | Constrains results to jobs that start with the defined prefix |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**per_page** | Option<**i32**> | Maximum number of results to return. |  |
**next_token** | Option<**String**> | Indicates where to start paging for queries that support pagination. |  |
**plugin_id** | Option<**String**> | Filters volume lists by plugin ID. |  |

### Return type

[**crate::models::CsiSnapshotListResponse**](CSISnapshotListResponse.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_volume

> crate::models::CsiVolume get_volume(volume_id, region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**volume_id** | **String** | Volume unique identifier. | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**index** | Option<**i32**> | If set, wait until query exceeds given index. Must be provided with WaitParam. |  |
**wait** | Option<**String**> | Provided with IndexParam to wait for change. |  |
**stale** | Option<**String**> | If present, results will include stale reads. |  |
**prefix** | Option<**String**> | Constrains results to jobs that start with the defined prefix |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**per_page** | Option<**i32**> | Maximum number of results to return. |  |
**next_token** | Option<**String**> | Indicates where to start paging for queries that support pagination. |  |

### Return type

[**crate::models::CsiVolume**](CSIVolume.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_volumes

> Vec<crate::models::CsiVolumeListStub> get_volumes(region, namespace, index, wait, stale, prefix, x_nomad_token, per_page, next_token, node_id, plugin_id, _type)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**index** | Option<**i32**> | If set, wait until query exceeds given index. Must be provided with WaitParam. |  |
**wait** | Option<**String**> | Provided with IndexParam to wait for change. |  |
**stale** | Option<**String**> | If present, results will include stale reads. |  |
**prefix** | Option<**String**> | Constrains results to jobs that start with the defined prefix |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**per_page** | Option<**i32**> | Maximum number of results to return. |  |
**next_token** | Option<**String**> | Indicates where to start paging for queries that support pagination. |  |
**node_id** | Option<**String**> | Filters volume lists by node ID. |  |
**plugin_id** | Option<**String**> | Filters volume lists by plugin ID. |  |
**_type** | Option<**String**> | Filters volume lists to a specific type. |  |

### Return type

[**Vec<crate::models::CsiVolumeListStub>**](CSIVolumeListStub.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_snapshot

> crate::models::CsiSnapshotCreateResponse post_snapshot(csi_snapshot_create_request, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**csi_snapshot_create_request** | [**CsiSnapshotCreateRequest**](CsiSnapshotCreateRequest.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

[**crate::models::CsiSnapshotCreateResponse**](CSISnapshotCreateResponse.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_volume

> post_volume(csi_volume_register_request, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**csi_volume_register_request** | [**CsiVolumeRegisterRequest**](CsiVolumeRegisterRequest.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

 (empty response body)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_volume_registration

> post_volume_registration(volume_id, csi_volume_register_request, region, namespace, x_nomad_token, idempotency_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**volume_id** | **String** | Volume unique identifier. | [required] |
**csi_volume_register_request** | [**CsiVolumeRegisterRequest**](CsiVolumeRegisterRequest.md) |  | [required] |
**region** | Option<**String**> | Filters results based on the specified region. |  |
**namespace** | Option<**String**> | Filters results based on the specified namespace. |  |
**x_nomad_token** | Option<**String**> | A Nomad ACL token. |  |
**idempotency_token** | Option<**String**> | Can be used to ensure operations are only run once. |  |

### Return type

 (empty response body)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

