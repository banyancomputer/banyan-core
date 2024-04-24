import { createAsyncThunk } from '@reduxjs/toolkit';

import { DealsClient } from '@/api/deals';
import { DealState } from '@/entities/deals';

const client = new DealsClient();

export const getAcceptedDeals = createAsyncThunk(
    'getAcceptedDeals',
    async () => await client.getDeals(DealState.Accepted)
);

export const getActiveDeals = createAsyncThunk(
    'getActiveDeals',
    async () => await client.getDeals(DealState.Active)
);

export const acceptDeal = createAsyncThunk(
    'acceptDeal',
    async (id: string) => await client.acceptDeal(id)
);

export const cancelDeal = createAsyncThunk(
    'cancelDeal',
    async (id: string) => await client.cancelDeal(id)
);

export const downloadDeal = createAsyncThunk(
    'downloadDeal',
    async (id: string) => await client.downloadDeal(id)
);
export const sealDeal = createAsyncThunk(
    'sealDeal',
    async (id: string) => await client.sealDeal(id)
);