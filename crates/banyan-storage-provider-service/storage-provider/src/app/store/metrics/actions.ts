import { MetricsClient } from "@/api/metrics";
import { createAsyncThunk } from "@reduxjs/toolkit";

const client = new MetricsClient();

export const getOveralStatistic = createAsyncThunk(
    'getOveralStatistic',
    async () => await client.getOveralStatistic()
);

export const getBandwidthUsage = createAsyncThunk(
    'getBandwidthUsage',
    async () => await client.getBandwidthUsage()
);

export const getStorageUsage = createAsyncThunk(
    'getStorageUsage',
    async () => await client.getStorageUsage()
);