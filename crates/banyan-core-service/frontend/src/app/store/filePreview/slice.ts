import { createSlice, PayloadAction } from "@reduxjs/toolkit";

import { BrowserObject, Bucket } from "@/app/types/bucket";
import { fileTypes, SUPPORTED_EXTENSIONS } from "@/app/types/filesPreview";
import { loadFilePreview } from "./actions";

interface OpenedFile {
    objectUrl: string;
    blob: File | null;
    fileType: string,
    isLoading: boolean;
    name: string;
    extension: string;
    browserObject: BrowserObject | null;
};

const initialOpenedFileState: OpenedFile = {
    objectUrl: '',
    name: '',
    blob: null,
    fileType: '',
    extension: '',
    isLoading: false,
    browserObject: null
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
    path: []
};

const filePreviewSlice = createSlice({
    name: 'filePreview',
    initialState: initialFilePreviewState,
    reducers: {
        closeFile(state) {
            Object.assign(state, initialFilePreviewState);
        },
        openFile(state, action: PayloadAction<{bucket: Bucket, file: BrowserObject, files: BrowserObject[], path: string[], parrentFolder?: BrowserObject}>){
            const {bucket, file, files, path, parrentFolder } = action.payload;
            const fileExtension = [...file.name.split('.')].pop() || '';
            Object.assign(state, { ...state, files, file: { ...initialOpenedFileState, name: file.name, browserObject: file, extension: fileExtension }, bucket, path, parrentFolder });
            SUPPORTED_EXTENSIONS.some((element, index) => {
                const result = element.includes(fileExtension);
                if(result) {
                    state.file.fileType = fileTypes[index];
                    return result;
                }
            });
        }
    },
    extraReducers(builder) {
        builder.addCase(loadFilePreview.pending, (state) => {
            state.file.isLoading = true;
        });
        builder.addCase(loadFilePreview.fulfilled, (state, action) => {
            state.file.objectUrl = action.payload;
            state.file.isLoading = false;
        });
    }
});

export const { closeFile, openFile  } = filePreviewSlice.actions;
export default filePreviewSlice.reducer;
