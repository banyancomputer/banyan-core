import { PayloadAction, createSlice } from '@reduxjs/toolkit';

export class BannerError {
    constructor(
        public message: string = '',
        public action: { label: string, callback: () => void } | null = null,
        public canBeClosed: boolean = true,
    ) { };
};

const initialState:BannerError[] = [];

const errorsSlice = createSlice({
    name: 'locales',
    initialState,
    reducers: {
        setError(state, action: PayloadAction<BannerError>) {
            state.push(action.payload);
        },

        closeError(state, action:PayloadAction<BannerError>) {
            return state.filter(error => error !== action.payload);
        }
    },
});

export const { setError, closeError } = errorsSlice.actions;

export default errorsSlice.reducer;
