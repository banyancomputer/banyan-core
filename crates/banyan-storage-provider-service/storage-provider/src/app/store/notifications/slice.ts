
import { createSlice } from "@reduxjs/toolkit";

import { getNotifications, getNotificationsHistory } from "./actions";
import { Notification } from "@/entities/notifications";


export interface NotificationsState {
    notifications: Notification[];
    notificationsHistory: Notification[];
};

const initialState:NotificationsState = {
    notifications: [],
    notificationsHistory:[]
};

const notificationsSlice = createSlice({
    name: 'notifications',
    initialState,
    reducers: { },
    extraReducers(builder) {
        builder.addCase(getNotifications.fulfilled, (state, action) => {
            state.notifications = action.payload;
        });

        builder.addCase(getNotificationsHistory.fulfilled, (state, action) => {
            state.notificationsHistory = action.payload;
        })
    },
});

export const { } = notificationsSlice.actions;
export default notificationsSlice.reducer;