import { createSlice, PayloadAction } from "@reduxjs/toolkit";

import { EscrowedKeyMaterial } from "@app/types";
import ECCKeystore from '@utils/crypto/ecc/keystore';
import { getEscrowedKeyMaterial, initializeKeystore } from "@app/store/keystore/actions";

export interface KeystoreState {
    keystore: ECCKeystore | null;
    escrowedKeyMaterial: EscrowedKeyMaterial | null;
    keystoreInitialized: boolean;
    isLoading: boolean;
	isLoggingOut: boolean;
};

const sessionSlice = createSlice({
    name: 'keystore',
    initialState: {
        keystore: null,
        escrowedKeyMaterial: null,
        keystoreInitialized: false,
        isLoading: true,
        isLoggingOut: false
    } as KeystoreState,
    reducers: {
        setEscrowedKeyMaterial(state, action:PayloadAction<EscrowedKeyMaterial>){
            state.escrowedKeyMaterial = action.payload;
        },
        setKeystore(state, action:PayloadAction<ECCKeystore>){
            state.keystore = action.payload;
        },
        setKeystoreInitialized(state, action:PayloadAction<boolean>){
            state.keystoreInitialized = action.payload;
        },
        setIsLoading(state, action:PayloadAction<boolean>){
            state.isLoading = action.payload;
        },
    },
    extraReducers(builder) {
        builder.addCase(getEscrowedKeyMaterial.fulfilled, (state, action) => {
            state.escrowedKeyMaterial = action.payload;
        });
        builder.addCase(initializeKeystore.fulfilled, (state, action) => {
            state.keystoreInitialized = true;
        });
    }
});

export const { setEscrowedKeyMaterial, setKeystore, setKeystoreInitialized, setIsLoading } = sessionSlice.actions;
export default sessionSlice.reducer;