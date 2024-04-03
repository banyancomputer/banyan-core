# \UsersApi

All URIs are relative to *https://beta.data.banyan.computer/api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_current_user**](UsersApi.md#get_current_user) | **GET** /users/current | Get current user information
[**get_escrowed_device**](UsersApi.md#get_escrowed_device) | **GET** /users/escrowed_device | Get current user's escrowed device
[**update_current_user**](UsersApi.md#update_current_user) | **PATCH** /users/current | Update current user information



## get_current_user

> models::ApiUser get_current_user()
Get current user information

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ApiUser**](ApiUser.md)

### Authorization

[SessionIdentity](../README.md#SessionIdentity), [ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_escrowed_device

> models::ApiEscrowedKeyMaterial get_escrowed_device()
Get current user's escrowed device

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ApiEscrowedKeyMaterial**](ApiEscrowedKeyMaterial.md)

### Authorization

[SessionIdentity](../README.md#SessionIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_current_user

> update_current_user(api_user)
Update current user information

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**api_user** | [**ApiUser**](ApiUser.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[SessionIdentity](../README.md#SessionIdentity), [ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

