import { MetricsClient } from "@/api/metrics";
import { createAsyncThunk } from "@reduxjs/toolkit";

const client = new MetricsClient();

export const getOverallStatistic = createAsyncThunk(
    'getOverallStatistic',
    async () => await client.getOverallStatistic()
);

export const getBandwidthUsage = createAsyncThunk(
    'getBandwidthUsage',
    async () => await client.getBandwidthUsage()
);

export const getStorageUsage = createAsyncThunk(
    'getStorageUsage',
    async () => await client.getStorageUsage()
);