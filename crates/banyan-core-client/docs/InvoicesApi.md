# \InvoicesApi

All URIs are relative to *https://beta.data.banyan.computer/api/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_all_invoices**](InvoicesApi.md#get_all_invoices) | **GET** /invoices | Get all invoices for the user
[**get_single_invoice**](InvoicesApi.md#get_single_invoice) | **GET** /invoices/{invoice_id} | Get a single invoice by ID



## get_all_invoices

> Vec<models::ApiInvoice> get_all_invoices()
Get all invoices for the user

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::ApiInvoice>**](ApiInvoice.md)

### Authorization

[SessionIdentity](../README.md#SessionIdentity), [ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_single_invoice

> models::ApiInvoice get_single_invoice(invoice_id)
Get a single invoice by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**invoice_id** | **uuid::Uuid** |  | [required] |

### Return type

[**models::ApiInvoice**](ApiInvoice.md)

### Authorization

[SessionIdentity](../README.md#SessionIdentity), [ApiIdentity](../README.md#ApiIdentity)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

