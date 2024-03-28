import { createAsyncThunk } from "@reduxjs/toolkit";

import { UserClient } from "@/api/user";

const client = new UserClient();

export const getUser = createAsyncThunk(
    'getUser',
    async () => {
        return await client.getCurrentUser();
    }
);
