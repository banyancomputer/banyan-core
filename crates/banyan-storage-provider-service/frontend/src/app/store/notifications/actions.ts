import { NotificationsClient } from "@/api/notifications";
import { createAsyncThunk } from "@reduxjs/toolkit";

const client = new NotificationsClient();

export const getNotifications = createAsyncThunk(
    'getNotifications',
    async () => await client.getNotifications()
);

export const getNotificationsHistory = createAsyncThunk(
    'getNotificationsHistory',
    async () => await client.getNotificationsHistory()
);