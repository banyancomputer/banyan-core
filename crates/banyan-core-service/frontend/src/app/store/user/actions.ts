import { createAsyncThunk } from "@reduxjs/toolkit";

import { UserClient } from "@/api/user";

const client = new UserClient();

export const getUserInfo = createAsyncThunk(
    'getUserInfo',
    async () => await client.getCurrentUser()
);