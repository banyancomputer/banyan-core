import { createSlice, PayloadAction } from "@reduxjs/toolkit";

import { User } from "@/entities/user";
import { getUser } from "./actions";

export interface SessionState {
	user: User;
};

const sessionSlice = createSlice({
    name: 'session',
    initialState: {
        user: {} as User,
    } as SessionState,
    reducers: {},
    extraReducers(builder) {
        builder.addCase(getUser.fulfilled, (state, action) => {
            state.user = action.payload;
        });
    }
});

export const { } = sessionSlice.actions;
export default sessionSlice.reducer;