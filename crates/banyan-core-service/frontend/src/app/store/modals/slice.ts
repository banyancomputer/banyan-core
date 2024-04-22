import { ReactNode } from 'react';
import { PayloadAction, createSlice } from '@reduxjs/toolkit';

export interface ModalsState {
    content: ReactNode | null;
    onBack: null | (() => void);
    mandatory: boolean;
    closeButton: boolean;
    className?: string;
};

const initialState: ModalsState = {
    content: null,
    onBack: null,
    mandatory: false,
    closeButton: true,
    className: ''
};

const modalsSlice = createSlice({
    name: 'modals',
    initialState,
    reducers: {
        openModal(state, action: PayloadAction<{content: ReactNode, onBack?: null | (() => void), mandatory?: boolean, className?: string, closeButton?: boolean}>) {
            Object.assign(state, action.payload);
        },

        closeModal(state) {
            Object.assign(state, initialState);
        },
    },
});

export const { openModal, closeModal } = modalsSlice.actions;

export default modalsSlice.reducer;
