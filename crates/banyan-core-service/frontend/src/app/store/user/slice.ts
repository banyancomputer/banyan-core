import { createSlice } from "@reduxjs/toolkit";

import { getUserInfo } from "./actions";
import { User } from "@/entities/user";

const userSlice = createSlice({
    name: 'user',
    initialState: {} as User,
    reducers: {},
    extraReducers(builder) {
        builder.addCase(getUserInfo.fulfilled, (state, action) => {
            Object.assign(state, action.payload);
        });
    }
});

export const { } = userSlice.actions;
export default userSlice.reducer;