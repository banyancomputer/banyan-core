import { ReactNode } from 'react';
import { PayloadAction, createSlice } from '@reduxjs/toolkit';

export interface ModalsState {
    content: ReactNode | null;
    onBack: null | (() => void);
    path: string[] | null
};

const initialState: ModalsState = {
    content: null,
    onBack: null,
    path: null
};

const modalsSlice = createSlice({
    name: 'modals',
    initialState,
    reducers: {
        openModal(state, action: PayloadAction<{ content: ReactNode, onBack?: null | (() => void), path?: string[] }>) {
            state.content = action.payload.content;
            state.onBack = action.payload.onBack ? action.payload.onBack : null;
            state.path = action.payload.path ? action.payload.path : null
        },

        closeModal(state) {
            Object.assign(state, initialState);
        },
    },
});

export const { openModal, closeModal } = modalsSlice.actions;

export default modalsSlice.reducer;
