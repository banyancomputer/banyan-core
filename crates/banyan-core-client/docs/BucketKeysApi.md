# \BucketKeysApi

All URIs are relative to *https://beta.data.banyan.computer/api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_bucket_key**](BucketKeysApi.md#create_bucket_key) | **POST** /buckets/{bucket_id}/keys | Create a new key for a specific bucket
[**delete_bucket_key**](BucketKeysApi.md#delete_bucket_key) | **DELETE** /buckets/{bucket_id}/keys/{bucket_key_id} | Delete a key for a specific bucket
[**get_all_bucket_keys**](BucketKeysApi.md#get_all_bucket_keys) | **GET** /buckets/{bucket_id}/keys | Get all keys for a specific bucket
[**get_single_bucket_key**](BucketKeysApi.md#get_single_bucket_key) | **GET** /buckets/{bucket_id}/keys/{bucket_key_id} | Get a single key for a specific bucket



## create_bucket_key

> models::ApiBucketKey create_bucket_key(bucket_id, create_bucket_key_request)
Create a new key for a specific bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**bucket_id** | **String** | The ID of the bucket to create a key for | [required] |
**create_bucket_key_request** | [**CreateBucketKeyRequest**](CreateBucketKeyRequest.md) |  | [required] |

### Return type

[**models::ApiBucketKey**](ApiBucketKey.md)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_bucket_key

> delete_bucket_key(bucket_id, bucket_key_id)
Delete a key for a specific bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**bucket_id** | **String** | The ID of the bucket the key belongs to | [required] |
**bucket_key_id** | **String** | The ID of the key to delete | [required] |

### Return type

 (empty response body)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_bucket_keys

> Vec<models::ApiBucketKey> get_all_bucket_keys(bucket_id)
Get all keys for a specific bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**bucket_id** | **String** | The ID of the bucket to get keys for | [required] |

### Return type

[**Vec<models::ApiBucketKey>**](ApiBucketKey.md)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_single_bucket_key

> models::ApiBucketKey get_single_bucket_key(bucket_id, bucket_key_id)
Get a single key for a specific bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**bucket_id** | **String** | The ID of the bucket the key belongs to | [required] |
**bucket_key_id** | **String** | The ID of the key to retrieve | [required] |

### Return type

[**models::ApiBucketKey**](ApiBucketKey.md)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

