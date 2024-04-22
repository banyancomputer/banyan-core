import { ReactNode } from 'react';
import { PayloadAction, createSlice } from '@reduxjs/toolkit';

export interface ModalsState {
    content: ReactNode | null;
    onBack: null | (() => void);
};

const initialState: ModalsState = {
    content: null,
    onBack: null,
};

const modalsSlice = createSlice({
    name: 'modals',
    initialState,
    reducers: {
        openModal(state, action: PayloadAction<{content: ReactNode, onBack?: null | (() => void)}>) {
            state.content = action.payload.content;
            state.onBack = action.payload.onBack? action.payload.onBack : null;
        },

        closeModal(state) {
            Object.assign(state, initialState);
        },
    },
});

export const { openModal, closeModal } = modalsSlice.actions;

export default modalsSlice.reducer;
