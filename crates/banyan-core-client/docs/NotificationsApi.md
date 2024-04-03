# \NotificationsApi

All URIs are relative to *https://beta.data.banyan.computer/api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_notification**](NotificationsApi.md#delete_notification) | **DELETE** /notifications/{notification_id} | Delete a notification by ID
[**get_all_notifications**](NotificationsApi.md#get_all_notifications) | **GET** /notifications | Get all notifications for the user



## delete_notification

> delete_notification(notification_id)
Delete a notification by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**notification_id** | **uuid::Uuid** |  | [required] |

### Return type

 (empty response body)

### Authorization

[SessionIdentity](../README.md#SessionIdentity), [ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_notifications

> Vec<models::ApiNotification> get_all_notifications()
Get all notifications for the user

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::ApiNotification>**](ApiNotification.md)

### Authorization

[SessionIdentity](../README.md#SessionIdentity), [ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

