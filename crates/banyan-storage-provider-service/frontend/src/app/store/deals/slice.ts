import { createSlice } from "@reduxjs/toolkit";

import { AvailableDeal, ActiveDeal } from "@/entities/deals";
import {getAvailableDeals, getAcceptedDeals} from "./actions";

export interface DealsState {
    availableDeals: AvailableDeal[];
    acceptedDeals: ActiveDeal[];
};

const initialState: DealsState = {
    acceptedDeals: [],
    availableDeals:[]
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

        builder.addCase(getAvailableDeals.fulfilled, (state, action) => {
            state.availableDeals = action.payload;
        });
    },
});

export const { } = dealsSlice.actions;
export default dealsSlice.reducer;