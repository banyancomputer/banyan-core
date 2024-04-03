# \BucketSnapshotsApi

All URIs are relative to *https://beta.data.banyan.computer/api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_all_snapshots**](BucketSnapshotsApi.md#get_all_snapshots) | **GET** /buckets/{bucket_id}/snapshots | Get all snapshots for a specific bucket
[**get_single_snapshot**](BucketSnapshotsApi.md#get_single_snapshot) | **GET** /buckets/{bucket_id}/snapshots/{snapshot_id} | Get a single snapshot for a specific bucket
[**restore_snapshot**](BucketSnapshotsApi.md#restore_snapshot) | **PUT** /buckets/{bucket_id}/snapshots/{snapshot_id}/restore | Restore a snapshot for a specific bucket



## get_all_snapshots

> Vec<models::ApiSnapshot> get_all_snapshots(bucket_id)
Get all snapshots for a specific bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**bucket_id** | **String** | The ID of the bucket to get snapshots for | [required] |

### Return type

[**Vec<models::ApiSnapshot>**](ApiSnapshot.md)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_single_snapshot

> models::ApiSnapshot get_single_snapshot(bucket_id, snapshot_id)
Get a single snapshot for a specific bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**bucket_id** | **String** | The ID of the bucket the snapshot belongs to | [required] |
**snapshot_id** | **String** | The ID of the snapshot to retrieve | [required] |

### Return type

[**models::ApiSnapshot**](ApiSnapshot.md)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## restore_snapshot

> models::RestoreSnapshot200Response restore_snapshot(bucket_id, snapshot_id)
Restore a snapshot for a specific bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**bucket_id** | **String** | The ID of the bucket the snapshot belongs to | [required] |
**snapshot_id** | **String** | The ID of the snapshot to restore | [required] |

### Return type

[**models::RestoreSnapshot200Response**](restoreSnapshot_200_response.md)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

