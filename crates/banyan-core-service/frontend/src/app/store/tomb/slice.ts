import { BrowserObject, Bucket } from "@/app/types/bucket";
import { StorageLimits, StorageUsage } from "@/entities/storage";
import { PayloadAction, createSlice } from "@reduxjs/toolkit";
import { TombWasm } from "tomb_build/banyanfs";
import { createBucketAndMount, getSelectedBucketSnapshots, getBuckets, getBucketsKeys, getExpandedFolderFiles, getSelectedBucketFiles, mountBucket, moveTo, renameBucket, createDirectory, updateStorageUsageState, updateStorageLimitsState, uploadFile, takeColdSnapshot } from "./actions";

interface TombState {
    tomb: TombWasm | null;
    buckets: Bucket[];
    trash: Bucket | null;
    selectedBucket: Bucket | null;
    storageUsage: StorageUsage;
    storageLimits: StorageLimits;
    isLoading: boolean;
    encryptionKey: {privatePem: string, publicPem: string } | null;
};

const initialState: TombState = {
    tomb: null,
    buckets: [],
    trash: null,
    selectedBucket: null,
    storageUsage: new StorageUsage(),
    storageLimits: new StorageLimits(),
    isLoading: true,
    encryptionKey: null
};

const tombSlice = createSlice({
    name: 'tomb',
    initialState,
    reducers: {
        setTomb(state, action: PayloadAction<TombWasm>) {
            state.tomb = action.payload;
        },
        selectBucket(state, action: PayloadAction<Bucket| null>){
            state.selectedBucket = action.payload;
        },
        setBucketFiles(state, action: PayloadAction<BrowserObject[]>) {
            state.selectedBucket!.files = action.payload;
        },
        updateBucketsState(state) {
            state.buckets = [...state.buckets];
        },
        setEncryptionKey(state, action: PayloadAction<{privatePem: string, publicPem: string } | null>) {
            state.encryptionKey = action.payload
        }
    },
    extraReducers(builder) {
        builder.addCase(getBuckets.fulfilled, (state, action) => {
            state.buckets = action.payload;
            state.isLoading = false;
        });
        builder.addCase(createBucketAndMount.fulfilled, (state, action) => {
            state.buckets = [...state.buckets, action.payload];
        });
        builder.addCase(mountBucket.fulfilled, (state, action) => {
            const bucket = state.buckets.find(bucket => bucket.id === action.payload.id)!;
            Object.assign(bucket, action.payload);
            if(bucket.id === state.selectedBucket?.id) {
                state.selectedBucket = {...bucket}
            };
            state.buckets = state.buckets.map(wasmBucket => wasmBucket.id === bucket.id ? bucket: wasmBucket);
        });
        builder.addCase(uploadFile.fulfilled, (state, action) => {
            if(action.payload.id === state.selectedBucket?.id) {
                Object.assign(state.selectedBucket!, action.payload);
            }
        });
        builder.addCase(getSelectedBucketFiles.fulfilled, (state, action) => {
            state.selectedBucket!.files = action.payload;
        });
        builder.addCase(getSelectedBucketSnapshots.fulfilled, (state, action) => {
            state.selectedBucket!.snapshots = action.payload;
        });
        builder.addCase(updateStorageUsageState.fulfilled, (state, action) => {
            state.storageUsage = action.payload;
        });
        builder.addCase(updateStorageLimitsState.fulfilled, (state, action) => {
            state.storageLimits = action.payload;
        });
    }
});

export const { selectBucket, setTomb, setBucketFiles, setEncryptionKey, updateBucketsState } = tombSlice.actions;
export default tombSlice.reducer;