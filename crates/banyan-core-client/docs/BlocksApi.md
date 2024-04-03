# \BlocksApi

All URIs are relative to *https://beta.data.banyan.computer/api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**locate_blocks**](BlocksApi.md#locate_blocks) | **POST** /blocks/locate | Locate blocks in storage



## locate_blocks

> std::collections::HashMap<String, Vec<String>> locate_blocks(request_body)
Locate blocks in storage

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**request_body** | [**Vec<String>**](String.md) |  | [required] |

### Return type

[**std::collections::HashMap<String, Vec<String>>**](Vec.md)

### Authorization

[ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

