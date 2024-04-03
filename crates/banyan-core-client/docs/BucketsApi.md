# \BucketsApi

All URIs are relative to *https://beta.data.banyan.computer/api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_bucket**](BucketsApi.md#create_bucket) | **POST** /buckets | Create a new bucket
[**create_bucket_key**](BucketsApi.md#create_bucket_key) | **POST** /buckets/{bucket_id}/keys | Create a new key for a specific bucket
[**delete_bucket**](BucketsApi.md#delete_bucket) | **DELETE** /buckets/{bucket_id} | Delete a single bucket by ID
[**delete_bucket_key**](BucketsApi.md#delete_bucket_key) | **DELETE** /buckets/{bucket_id}/keys/{bucket_key_id} | Delete a key for a specific bucket
[**delete_bucket_metadata**](BucketsApi.md#delete_bucket_metadata) | **DELETE** /buckets/metadata/{metadata_id} | Delete a metadata entry by its ID
[**get_all_bucket_keys**](BucketsApi.md#get_all_bucket_keys) | **GET** /buckets/{bucket_id}/keys | Get all keys for a specific bucket
[**get_all_bucket_metadata**](BucketsApi.md#get_all_bucket_metadata) | **GET** /buckets/metadata | Retrieve all metadata associated with the user's account
[**get_all_buckets**](BucketsApi.md#get_all_buckets) | **GET** /buckets | Get all buckets for the user
[**get_all_snapshots**](BucketsApi.md#get_all_snapshots) | **GET** /buckets/{bucket_id}/snapshots | Get all snapshots for a specific bucket
[**get_bucket_authorization_grants**](BucketsApi.md#get_bucket_authorization_grants) | **GET** /buckets/{bucket_id}/authorization_grants | Get authorization grants for a specific bucket
[**get_bucket_usage**](BucketsApi.md#get_bucket_usage) | **GET** /buckets/{bucket_id}/usage | Get the usage information for a specific bucket
[**get_current_bucket_metadata**](BucketsApi.md#get_current_bucket_metadata) | **GET** /buckets/metadata/current | Retrieve the current metadata for a bucket
[**get_single_bucket**](BucketsApi.md#get_single_bucket) | **GET** /buckets/{bucket_id} | Get a single bucket by ID
[**get_single_bucket_key**](BucketsApi.md#get_single_bucket_key) | **GET** /buckets/{bucket_id}/keys/{bucket_key_id} | Get a single key for a specific bucket
[**get_single_bucket_metadata**](BucketsApi.md#get_single_bucket_metadata) | **GET** /buckets/metadata/{metadata_id} | Retrieve a single metadata entry by its ID
[**get_single_snapshot**](BucketsApi.md#get_single_snapshot) | **GET** /buckets/{bucket_id}/snapshots/{snapshot_id} | Get a single snapshot for a specific bucket
[**pull_bucket_metadata**](BucketsApi.md#pull_bucket_metadata) | **GET** /buckets/metadata/{metadata_id}/pull | Pull metadata by its ID
[**push_bucket_metadata**](BucketsApi.md#push_bucket_metadata) | **POST** /buckets/metadata | Push new metadata to a bucket
[**restore_snapshot**](BucketsApi.md#restore_snapshot) | **PUT** /buckets/{bucket_id}/snapshots/{snapshot_id}/restore | Restore a snapshot for a specific bucket
[**snapshot_bucket_metadata**](BucketsApi.md#snapshot_bucket_metadata) | **POST** /buckets/metadata/{metadata_id}/snapshot | Create a snapshot for the metadata by its ID
[**update_bucket**](BucketsApi.md#update_bucket) | **PUT** /buckets/{bucket_id} | Update a single bucket by ID



## create_bucket

> models::CreateBucketResponse create_bucket(create_bucket_request)
Create a new bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_bucket_request** | [**CreateBucketRequest**](CreateBucketRequest.md) |  | [required] |

### Return type

[**models::CreateBucketResponse**](CreateBucketResponse.md)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


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


## delete_bucket

> delete_bucket(bucket_id)
Delete a single bucket by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**bucket_id** | **String** | The unique identifier for the bucket to be deleted | [required] |

### Return type

 (empty response body)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
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


## get_all_buckets

> Vec<models::ApiBucket> get_all_buckets()
Get all buckets for the user

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::ApiBucket>**](ApiBucket.md)

### Authorization

[SessionIdentity](../README.md#SessionIdentity), [ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


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


## get_bucket_authorization_grants

> models::AuthorizationGrant get_bucket_authorization_grants(bucket_id)
Get authorization grants for a specific bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**bucket_id** | **String** | The ID of the bucket to get authorization grants for | [required] |

### Return type

[**models::AuthorizationGrant**](AuthorizationGrant.md)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_bucket_usage

> models::BucketUsage get_bucket_usage(bucket_id)
Get the usage information for a specific bucket

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**bucket_id** | **String** | The ID of the bucket to get usage information for | [required] |

### Return type

[**models::BucketUsage**](BucketUsage.md)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

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


## get_single_bucket

> models::ApiBucket get_single_bucket(bucket_id)
Get a single bucket by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**bucket_id** | **String** | The unique identifier for the bucket | [required] |

### Return type

[**models::ApiBucket**](ApiBucket.md)

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


## snapshot_bucket_metadata

> models::ApiMetadata snapshot_bucket_metadata(metadata_id)
Create a snapshot for the metadata by its ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**metadata_id** | **String** | The ID of the metadata to snapshot | [required] |

### Return type

[**models::ApiMetadata**](ApiMetadata.md)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_bucket

> update_bucket(bucket_id, api_bucket_configuration)
Update a single bucket by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**bucket_id** | **String** | The unique identifier for the bucket to be updated | [required] |
**api_bucket_configuration** | [**ApiBucketConfiguration**](ApiBucketConfiguration.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

