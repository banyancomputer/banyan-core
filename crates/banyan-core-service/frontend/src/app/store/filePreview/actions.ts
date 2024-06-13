import { createAsyncThunk, unwrapResult } from "@reduxjs/toolkit";
import mime from 'mime';
import { RootState } from "@store/index";
import { getFile } from "../tomb/actions";

export const loadFilePreview = createAsyncThunk(
    'loadFilePreview',
    async(_, { dispatch, getState }) => {
        const { filePreview: {file, bucket,  path} } = getState() as RootState;
        const arrayBuffer = unwrapResult(await dispatch(getFile({bucket: bucket!, path, name: file.name})));
        const blob = new File([arrayBuffer], file.name, { type: mime.getType(file.extension) || '' });
        return URL.createObjectURL(blob);
    }
);
