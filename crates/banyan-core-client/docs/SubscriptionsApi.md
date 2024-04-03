# \SubscriptionsApi

All URIs are relative to *https://beta.data.banyan.computer/api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_all_subscriptions**](SubscriptionsApi.md#get_all_subscriptions) | **GET** /subscriptions | Get all subscriptions
[**get_single_subscription**](SubscriptionsApi.md#get_single_subscription) | **GET** /subscriptions/{subscription_id} | Get a single subscription by ID
[**manage_subscription**](SubscriptionsApi.md#manage_subscription) | **GET** /subscriptions/manage | Manage the subscription
[**purchase_subscription**](SubscriptionsApi.md#purchase_subscription) | **POST** /subscriptions/{subscription_id}/subscribe | Purchase a subscription
[**subscription_purchase_cancel**](SubscriptionsApi.md#subscription_purchase_cancel) | **GET** /subscriptions/cancel | Subscription purchase cancel redirect
[**subscription_purchase_success**](SubscriptionsApi.md#subscription_purchase_success) | **GET** /subscriptions/success/{checkout_session_id} | Subscription purchase success redirect



## get_all_subscriptions

> Vec<models::ApiSubscription> get_all_subscriptions()
Get all subscriptions

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::ApiSubscription>**](ApiSubscription.md)

### Authorization

[SessionIdentity](../README.md#SessionIdentity), [ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_single_subscription

> models::ApiSubscription get_single_subscription(subscription_id)
Get a single subscription by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**subscription_id** | **String** |  | [required] |

### Return type

[**models::ApiSubscription**](ApiSubscription.md)

### Authorization

[SessionIdentity](../README.md#SessionIdentity), [ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## manage_subscription

> models::ManageSubscription200Response manage_subscription()
Manage the subscription

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ManageSubscription200Response**](manageSubscription_200_response.md)

### Authorization

[SessionIdentity](../README.md#SessionIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## purchase_subscription

> models::PurchaseSubscription200Response purchase_subscription(subscription_id)
Purchase a subscription

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**subscription_id** | **String** |  | [required] |

### Return type

[**models::PurchaseSubscription200Response**](purchaseSubscription_200_response.md)

### Authorization

[SessionIdentity](../README.md#SessionIdentity), [ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## subscription_purchase_cancel

> subscription_purchase_cancel()
Subscription purchase cancel redirect

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

[SessionIdentity](../README.md#SessionIdentity), [ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## subscription_purchase_success

> subscription_purchase_success(checkout_session_id)
Subscription purchase success redirect

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**checkout_session_id** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[SessionIdentity](../README.md#SessionIdentity), [ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

