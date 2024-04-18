import { createSlice } from "@reduxjs/toolkit";

import { BandwidthUsage, OveralStatistic, StorageUsage } from "@/entities/metrics";
import { getBandwidthUsage, getOverallStatistic, getStorageUsage } from "./actions";

export interface MetricsState {
    storageUsage: StorageUsage[];
    bandWidthUsage: BandwidthUsage[];
    overalStatistic: OveralStatistic;
};

const initialState: MetricsState = {
    storageUsage: [],
    bandWidthUsage:[],
    overalStatistic: {} as OveralStatistic
};

const metricsSlice = createSlice({
    name: 'metrics',
    initialState,
    reducers: {},
    extraReducers(builder) {
        builder.addCase(getOverallStatistic.fulfilled, (state, action) =>{
            state.overalStatistic = action.payload;
        });
        builder.addCase(getBandwidthUsage.fulfilled, (state, action) =>{
            state.bandWidthUsage = action.payload;
        });
        builder.addCase(getStorageUsage.fulfilled, (state, action) =>{
            state.storageUsage = action.payload
        });
    },
});

export default metricsSlice.reducer;