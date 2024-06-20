import { PayloadAction, createSlice } from '@reduxjs/toolkit';

import { loadFilePreview } from './actions';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { SUPPORTED_FILE_TYPES } from '@/app/types/filesPreview';

interface OpenedFile {
    objectUrl: string;
    blob: File | null;
    fileType: string;
    isLoading: boolean;
    name: string;
    mimeType: string;
    browserObject: BrowserObject | null;
};

const initialOpenedFileState: OpenedFile = {
    objectUrl: '',
    name: '',
    blob: null,
    fileType: '',
    mimeType: '',
    isLoading: false,
    browserObject: null,
};

export interface FilePreviewState {
    file: OpenedFile;
    files: BrowserObject[];
    bucket: Bucket | null;
    path: string[];
    parrentFolder: BrowserObject | undefined;
};

export const initialFilePreviewState: FilePreviewState = {
    file: initialOpenedFileState,
    files: [],
    bucket: null,
    parrentFolder: undefined,
    path: [],
};

const filePreviewSlice = createSlice({
    name: 'filePreview',
    initialState: initialFilePreviewState,
    reducers: {
        closeFile(state) {
            Object.assign(state, initialFilePreviewState);
        },
        openFile(state, action: PayloadAction<{bucket: Bucket; file: BrowserObject; files: BrowserObject[]; path: string[]; parrentFolder?: BrowserObject}>) {
            const { bucket, file, files, path, parrentFolder } = action.payload;
            Object.assign(state, { ...state, files, file: { ...initialOpenedFileState, name: file.name, browserObject: file, mimeType: file.metadata.mime || '' }, bucket, path, parrentFolder });
            SUPPORTED_FILE_TYPES.some(element => {
                const result = element.mimeTypes.includes(file.metadata.mime || '');
                if(result) {
                    state.file.fileType = element.type;

                    return result;
                }
            });
        },
    },
    extraReducers(builder) {
        builder.addCase(loadFilePreview.pending, (state) => {
            state.file.isLoading = true;
        });
        builder.addCase(loadFilePreview.fulfilled, (state, action) => {
            state.file.objectUrl = action.payload;
            state.file.isLoading = false;
        });
    },
});

export const { closeFile, openFile } = filePreviewSlice.actions;
export default filePreviewSlice.reducer;
