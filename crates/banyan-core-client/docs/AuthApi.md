# \AuthApi

All URIs are relative to *https://beta.data.banyan.computer/api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_device_api_key**](AuthApi.md#create_device_api_key) | **POST** /auth/device_api_key | Create a new device API key
[**delete_device_api_key**](AuthApi.md#delete_device_api_key) | **DELETE** /auth/device_api_key/{key_id} | Delete a specific device API key
[**read_all_device_api_keys**](AuthApi.md#read_all_device_api_keys) | **GET** /auth/device_api_key | Get all device API keys for the user
[**read_device_api_key**](AuthApi.md#read_device_api_key) | **GET** /auth/device_api_key/{key_id} | Read a specific device API key
[**who_am_i**](AuthApi.md#who_am_i) | **GET** /auth/who_am_i | Get the user ID of the currently authenticated user



## create_device_api_key

> models::CreateDeviceApiKey200Response create_device_api_key(create_device_api_key_request)
Create a new device API key

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_device_api_key_request** | [**CreateDeviceApiKeyRequest**](CreateDeviceApiKeyRequest.md) |  | [required] |

### Return type

[**models::CreateDeviceApiKey200Response**](createDeviceApiKey_200_response.md)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_device_api_key

> delete_device_api_key(key_id)
Delete a specific device API key

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**key_id** | **uuid::Uuid** |  | [required] |

### Return type

 (empty response body)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## read_all_device_api_keys

> Vec<models::DeviceApiKey> read_all_device_api_keys()
Get all device API keys for the user

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::DeviceApiKey>**](DeviceApiKey.md)

### Authorization

[SessionIdentity](../README.md#SessionIdentity), [ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## read_device_api_key

> models::DeviceApiKey read_device_api_key(key_id)
Read a specific device API key

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**key_id** | **uuid::Uuid** |  | [required] |

### Return type

[**models::DeviceApiKey**](DeviceApiKey.md)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## who_am_i

> models::WhoAmI200Response who_am_i()
Get the user ID of the currently authenticated user

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::WhoAmI200Response**](whoAmI_200_response.md)

### Authorization

[SessionIdentity](../README.md#SessionIdentity), [ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

