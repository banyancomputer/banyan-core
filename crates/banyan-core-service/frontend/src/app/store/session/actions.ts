import { createAsyncThunk } from "@reduxjs/toolkit";

import { UserClient } from "@/api/user";

const client = new UserClient();

export const getUser = createAsyncThunk(
    'getUser',
    async () => {
        return await client.getCurrentUser();
    }
);

    export const getEscrowedKeyMaterial = createAsyncThunk(
        'getEscrowedKeyMaterial',
        async () => {
        return await client.getEscrowedKeyMaterial();
    }
);