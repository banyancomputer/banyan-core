import { createSlice } from "@reduxjs/toolkit";

import { AvailiableDeal, ActiveDeal } from "@/entities/deals";
import { getActiceDeals, getAvailableDeals } from "./actions";

export interface DealsState {
    availiableDeals: AvailiableDeal[];
    activeDeals: ActiveDeal[];
};

const initialState: DealsState = {
    activeDeals: [],
    availiableDeals:[]
}

const dealsSlice = createSlice({
    name: 'deals',
    initialState,
    reducers: {

    },
    extraReducers(builder) {
        builder.addCase(getActiceDeals.fulfilled, (state, action) => {
            state.activeDeals = action.payload;
        });

        builder.addCase(getAvailableDeals.fulfilled, (state, action) => {
            state.availiableDeals = action.payload;
        });
    },
});

export const { } = dealsSlice.actions;
export default dealsSlice.reducer;