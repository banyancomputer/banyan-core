import { PayloadAction, createSlice } from "@reduxjs/toolkit";

type FileUploadingStatus = "pending" | "uploading" | "success" | "failed";
export interface UploadingFile { file: File; status: FileUploadingStatus };

interface FilesUploadState {
    files: UploadingFile[]
};

export const initialState: FilesUploadState = {
    files: []
};

const filesUploadSlice = createSlice({
    name: 'fileUpload',
    initialState,
    reducers: {
        setFiles(state, action: PayloadAction<UploadingFile[]>) {
            state.files = action.payload;
            console.log('setFiles', state.files);
        },
        updateFileStatus(state, action: PayloadAction<{file: UploadingFile, status: FileUploadingStatus}>) {
            const {file, status} = action.payload;
            state.files = state.files.map(uploadingFile => uploadingFile.file.name === file.file.name? {...uploadingFile, status: status} : uploadingFile);
        },
        deleteFile(state, action: PayloadAction<UploadingFile>) {
            state.files = state.files.filter(file => file !== action.payload);
        }
    }
});

export const { setFiles, updateFileStatus, deleteFile } = filesUploadSlice.actions;
export default filesUploadSlice.reducer;
