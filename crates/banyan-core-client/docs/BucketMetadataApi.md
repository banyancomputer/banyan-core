# \BucketMetadataApi

All URIs are relative to *https://beta.data.banyan.computer/api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_bucket_metadata**](BucketMetadataApi.md#delete_bucket_metadata) | **DELETE** /buckets/metadata/{metadata_id} | Delete a metadata entry by its ID
[**get_all_bucket_metadata**](BucketMetadataApi.md#get_all_bucket_metadata) | **GET** /buckets/metadata | Retrieve all metadata associated with the user's account
[**get_current_bucket_metadata**](BucketMetadataApi.md#get_current_bucket_metadata) | **GET** /buckets/metadata/current | Retrieve the current metadata for a bucket
[**get_single_bucket_metadata**](BucketMetadataApi.md#get_single_bucket_metadata) | **GET** /buckets/metadata/{metadata_id} | Retrieve a single metadata entry by its ID
[**pull_bucket_metadata**](BucketMetadataApi.md#pull_bucket_metadata) | **GET** /buckets/metadata/{metadata_id}/pull | Pull metadata by its ID
[**push_bucket_metadata**](BucketMetadataApi.md#push_bucket_metadata) | **POST** /buckets/metadata | Push new metadata to a bucket



## delete_bucket_metadata

> delete_bucket_metadata(metadata_id)
Delete a metadata entry by its ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**metadata_id** | **String** | The ID of the metadata to delete | [required] |

### Return type

 (empty response body)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_bucket_metadata

> Vec<models::ApiMetadata> get_all_bucket_metadata()
Retrieve all metadata associated with the user's account

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::ApiMetadata>**](ApiMetadata.md)

### Authorization

[SessionIdentity](../README.md#SessionIdentity), [ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_current_bucket_metadata

> models::ApiMetadata get_current_bucket_metadata()
Retrieve the current metadata for a bucket

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ApiMetadata**](ApiMetadata.md)

### Authorization

[SessionIdentity](../README.md#SessionIdentity), [ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_single_bucket_metadata

> models::ApiMetadata get_single_bucket_metadata(metadata_id)
Retrieve a single metadata entry by its ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**metadata_id** | **String** | The ID of the metadata to retrieve | [required] |

### Return type

[**models::ApiMetadata**](ApiMetadata.md)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## pull_bucket_metadata

> std::path::PathBuf pull_bucket_metadata(metadata_id)
Pull metadata by its ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**metadata_id** | **String** | The ID of the metadata to pull | [required] |

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/octet-stream, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## push_bucket_metadata

> models::ApiMetadataResponse push_bucket_metadata(request_data, car_upload)
Push new metadata to a bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**request_data** | **std::path::PathBuf** | JSON payload that precedes the metadata CAR file upload | [required] |
**car_upload** | **std::path::PathBuf** | The CAR file containing the metadata to be uploaded | [required] |

### Return type

[**models::ApiMetadataResponse**](ApiMetadataResponse.md)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

