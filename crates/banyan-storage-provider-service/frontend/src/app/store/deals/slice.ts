import { createSlice } from "@reduxjs/toolkit";

import { AvailiableDeal, ActiveDeal } from "@/entities/deals";
import { getAcceptedDeals, getAvailableDeals } from "./actions";

export interface DealsState {
    availableDeals: AvailiableDeal[];
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