import { createSlice, PayloadAction } from "@reduxjs/toolkit";

import { EscrowedKeyMaterial } from "@app/types/escrowedKeyMaterial";
import ECCKeystore from '@utils/crypto/ecc/keystore';
import { getEscrowedKeyMaterial, initializeKeystore } from "@store/keystore/actions";

export interface KeystoreState {
    keystore: ECCKeystore | null;
    escrowedKeyMaterial: EscrowedKeyMaterial | null;
    keystoreInitialized: boolean;
};

const keystoreSlice = createSlice({
    name: 'keystore',
    initialState: {
        keystore: null,
        escrowedKeyMaterial: null,
        keystoreInitialized: false,
    } as KeystoreState,
    reducers: {
        setEscrowedKeyMaterial(state, action: PayloadAction<EscrowedKeyMaterial>) {
            state.escrowedKeyMaterial = action.payload;
        },
        setKeystore(state, action: PayloadAction<ECCKeystore>) {
            state.keystore = action.payload;
        },
        setKeystoreInitialized(state, action: PayloadAction<boolean>) {
            state.keystoreInitialized = action.payload;
        },
    },
    extraReducers(builder) {
        builder.addCase(getEscrowedKeyMaterial.fulfilled, (state, action) => {
            state.escrowedKeyMaterial = action.payload;
        });
        builder.addCase(getEscrowedKeyMaterial.rejected, (state) => {
            state.keystoreInitialized = false;
        });
        builder.addCase(initializeKeystore.fulfilled, (state) => {
            state.keystoreInitialized = true;
        });
    }
});

export const { setEscrowedKeyMaterial, setKeystore, setKeystoreInitialized } = keystoreSlice.actions;
export default keystoreSlice.reducer;
