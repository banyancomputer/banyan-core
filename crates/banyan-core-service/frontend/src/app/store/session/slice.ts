import { createSlice, PayloadAction } from "@reduxjs/toolkit";

import { User } from "@/entities/user";
import { EscrowedKeyMaterial } from "@/app/types";
import { getEscrowedKeyMaterial, getUser } from "./actions";

export interface SessionState {
	user: User,
    escrowedKeyMaterial: EscrowedKeyMaterial | null
};

const sessionSlice = createSlice({
    name: 'session',
    initialState: {
        user: {} as User,
        escrowedKeyMaterial: null
    } as SessionState,
    reducers: {
        setEscrowedKeyMaterial(state, action:PayloadAction<EscrowedKeyMaterial>){
            state.escrowedKeyMaterial = action.payload;
        }
    },
    extraReducers(builder) {
        builder.addCase(getUser.fulfilled, (state, action) => {
            state.user = action.payload;
        });
        builder.addCase(getEscrowedKeyMaterial.fulfilled, (state, action) => {
            state.escrowedKeyMaterial = action.payload;
        });
    }
});

export const { setEscrowedKeyMaterial} = sessionSlice.actions;
export default sessionSlice.reducer;