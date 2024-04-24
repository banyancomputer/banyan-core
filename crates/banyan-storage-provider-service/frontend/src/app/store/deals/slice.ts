import { createSlice } from "@reduxjs/toolkit";

import { ActiveDeal, AcceptedDeal } from "@/entities/deals";
import {getActiveDeals, getAcceptedDeals} from "./actions";

export interface DealsState {
    activeDeals: ActiveDeal[];
    acceptedDeals: AcceptedDeal[];
};

const initialState: DealsState = {
    acceptedDeals: [],
    activeDeals: []
}

const dealsSlice = createSlice({
    name: 'deals',
    initialState,
    reducers: {

    },
    extraReducers(builder) {
        builder.addCase(getAcceptedDeals.fulfilled, (state, action) => {
            state.acceptedDeals = action.payload;
        });

        builder.addCase(getActiveDeals.fulfilled, (state, action) => {
            state.activeDeals = action.payload;
        });
    },
});

export const { } = dealsSlice.actions;
export default dealsSlice.reducer;