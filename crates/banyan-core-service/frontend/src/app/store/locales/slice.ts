import { PayloadAction, createSlice } from '@reduxjs/toolkit';
import en from '@static/locales/en';

type LocaleType = typeof en;

export const LANGUAGES: Record<string, LocaleType> = {
    en,
};

export type LANGUAGES_KEYS = keyof typeof LANGUAGES;

export interface LocalesState {
    messages: LocaleType;
    key: LANGUAGES_KEYS;
};

const initialState: LocalesState = {
    messages: en,
    key: 'en',
};

const modalsSlice = createSlice({
    name: 'locales',
    initialState,
    reducers: {
        changeLanguage(state, action: PayloadAction<LANGUAGES_KEYS>) {
            state.messages = LANGUAGES[action.payload];
            state.key = action.payload;
        },
    },
});

export const { changeLanguage } = modalsSlice.actions;

export default modalsSlice.reducer;
