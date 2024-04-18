import { createAsyncThunk } from "@reduxjs/toolkit";

import { DealsClient } from "@/api/deals";

const client = new DealsClient();

export const getAcceptedDeals = createAsyncThunk(
    'getAcceptedDeals',
    async () => await client.getDeals("accepted")
);

export const getAvailableDeals = createAsyncThunk(
    'getAvailableDeals',
    async () => await client.getDeals()
);

export const acceptDeal = createAsyncThunk(
    'acceptDeal',
    async (id: string) => await client.acceptDeal(id)
);

export const rejectDeal = createAsyncThunk(
    'rejectDeal',
    async (id: string) => await client.rejectDeal(id)
);

export const downloadDeal = createAsyncThunk(
    'downloadDeal',
    async (id: string) => await client.downloadDeal(id)
);
export const proofDeal = createAsyncThunk(
    'proofDeal',
    async (id: string) => await client.proofDeal(id)
);