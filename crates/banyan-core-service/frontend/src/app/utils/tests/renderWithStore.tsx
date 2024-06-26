import React, { ReactElement } from "react";
import { render } from "@testing-library/react";
import { combineReducers, configureStore } from "@reduxjs/toolkit";
import { Provider } from "react-redux";
// As a basic setup, import your same slice reducers
import tomb from "@store/tomb/slice";
import locales from "@store/locales/slice";
import { initialState, TombState } from "@store/tomb/slice";

const rootReducer = combineReducers({
    tomb,
    locales,
})

export function renderWithProviders(
    ui: ReactElement,
    {
        // Automatically create a store instance if no store was passed in
        store = configureStore({
            reducer: rootReducer,
        }),
        ...renderOptions
    } = {}
) {
    function Wrapper({ children }: { children: ReactElement }) {
        return <Provider store={store}>{children}</Provider>;
    }

    // Return an object with the store and all of RTL's query functions
    return { store, ...render(ui, { wrapper: Wrapper, ...renderOptions }) };
}