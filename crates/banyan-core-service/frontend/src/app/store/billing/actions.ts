import { BillingClient } from "@/api/billing";
import { createAsyncThunk } from "@reduxjs/toolkit";

const client = new BillingClient();

export const getInvoices = createAsyncThunk(
    'getInvoices',
    async () => client.getInvoices()
);

export const getInvoiceById = createAsyncThunk(
    'getInvoiceById',
    async (id: string) => client.getInvoiceById(id)
);

export const getSubscriptions = createAsyncThunk(
    'getSubscriptions',
    async () => client.getSubscriptions()
);

export const getSubscriptionById = createAsyncThunk(
    'getSubscriptionById',
    async (id: string) => client.getSubscriptionById(id)
);

export const subscribe = createAsyncThunk(
    'subscribe',
    async (id: string) => client.subscribe(id)
);

export const manageSubscriptions = createAsyncThunk(
    'manageSubscriptions',
    async () => client.manage()
);
