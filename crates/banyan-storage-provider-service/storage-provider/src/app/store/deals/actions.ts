import { createAsyncThunk } from "@reduxjs/toolkit";

import { DealsClient } from "@/api/deals";

const client = new DealsClient();

export const getActiceDeals = createAsyncThunk(
    'getActiceDeals',
    async () => await client.getActiceDeals()
);

export const getAvailableDeals = createAsyncThunk(
    'getAvailableDeals',
    async () => await client.getAvailableDeals()
);

export const acceptDeal = createAsyncThunk(
    'acceptDeal',
    async (id: string) => await client.acceptDeal(id)
);

export const declineDeal = createAsyncThunk(
    'declineDeal',
    async (id: string) => await client.declineDeal(id)
);

export const downloadDeal = createAsyncThunk(
    'downloadDeal',
    async (id: string) => await client.downloadDeal(id)
);
export const proofDeal = createAsyncThunk(
    'proofDeal',
    async (id: string) => await client.proofDeal(id)
);