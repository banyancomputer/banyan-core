import { BrowserObject, Bucket } from "@/app/types/bucket";
import { StorageLimits, StorageUsage } from "@/entities/storage";
import { PayloadAction, createSlice } from "@reduxjs/toolkit";
import { TombWasm } from "tomb_build/banyanfs";
import { createBucketAndMount, getSelectedBucketSnapshots, deleteFile, getBuckets, getExpandedFolderFiles, getSelectedBucketFiles, mountBucket, moveTo, renameBucket, createDirectory, updateStorageUsageState, updateStorageLimitsState, uploadFile, takeColdSnapshot } from "./actions";
import { UploadWorker } from "@/workers/upload.worker";
import { Remote } from "comlink";

export interface TombState {
    tomb: TombWasm | null;
    buckets: Bucket[];
    trash: Bucket | null;
    selectedBucket: Bucket | null;
    storageUsage: StorageUsage;
    storageLimits: StorageLimits;
    isLoading: boolean;
    encryptionKey: {privatePem: string, publicPem: string } | null;
    worker: Remote<UploadWorker> | null;
};

export const initialState: TombState = {
    tomb: null,
    buckets: [],
    trash: null,
    selectedBucket: null,
    storageUsage: new StorageUsage(),
    storageLimits: new StorageLimits(),
    isLoading: true,
    encryptionKey: null,
    worker: null
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
        },
        setIsLoading(state, action: PayloadAction<boolean>) {
            state.isLoading = action.payload
        },
        setWorker(state, action: PayloadAction<Remote<UploadWorker>>) {
            state.worker = action.payload;
        },
        setBuckets(state, action: PayloadAction<Bucket[]>) {
            state.buckets = action.payload;
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
            const { id, mount, isSnapshotValid, locked } = action.payload;
            if(id === state.selectedBucket?.id) {
                state.selectedBucket = {...state.selectedBucket, mount, isSnapshotValid, locked};
            };

            state.buckets = state.buckets.map(wasmBucket =>
                wasmBucket.id === action.payload.id ?
                {...wasmBucket, mount, isSnapshotValid, locked}
                :
                wasmBucket
            );
        });
        builder.addCase(uploadFile.fulfilled, (state, action) => {
            if(action.payload.id === state.selectedBucket?.id) {
                Object.assign(state.selectedBucket!, action.payload);
            };
        });
        builder.addCase(getExpandedFolderFiles.fulfilled, (state) => {
            state.selectedBucket!.files = [...state.selectedBucket!.files];
        });
        builder.addCase(takeColdSnapshot.fulfilled, (state) => {
            if(state.selectedBucket) {
                state.selectedBucket = {...state.selectedBucket}
            };
            state.buckets = [...state.buckets];
        });
        builder.addCase(getSelectedBucketFiles.pending, (state) => {
            if(!state.selectedBucket?.files.length) {
                state.isLoading = true;
            };
        });
        builder.addCase(getSelectedBucketFiles.fulfilled, (state, action) => {
            state.selectedBucket!.files = action.payload;
            state.isLoading = false;
        });
        builder.addCase(getSelectedBucketSnapshots.fulfilled, (state, action) => {
            state.selectedBucket!.snapshots = action.payload;
        });
        builder.addCase(updateStorageUsageState.fulfilled, (state, action) => {
            state.storageUsage = action.payload;
        });
        builder.addCase(createDirectory.fulfilled, (state, action) => {
            const files = action.payload?.files;
            const id = action.payload?.id;
            if(files) {
                if(state.selectedBucket) {
                    state.selectedBucket!.files = action.payload!.files;
                    state.selectedBucket!.isSnapshotValid = false;
                };
                state.buckets = state.buckets.map(wasmBucket => wasmBucket.id === id ? {...wasmBucket, files, isSnapshotValid: false } : wasmBucket);
        }
        });
        builder.addCase(moveTo.fulfilled, (state) => {
            state.selectedBucket!.isSnapshotValid = false;
        });
        builder.addCase(updateStorageLimitsState.fulfilled, (state, action) => {
            state.storageLimits = action.payload;
        });
        builder.addCase(deleteFile.fulfilled, (state) => {
            state.selectedBucket!.isSnapshotValid = false;
        });
        builder.addCase(renameBucket.fulfilled, (state, action) => {
            if(state.selectedBucket) {
                state.selectedBucket.name = action.payload.name;
            };

            state.buckets = state.buckets.map(wasmBucket => wasmBucket.id === action.payload.bucketId ? {...wasmBucket, name: action.payload.name } : wasmBucket);
        });
    }
});

export const { selectBucket, setBuckets, setTomb, setWorker, setBucketFiles, setIsLoading, setEncryptionKey, updateBucketsState } = tombSlice.actions;
export default tombSlice.reducer;