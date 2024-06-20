import { createAsyncThunk, unwrapResult } from '@reduxjs/toolkit';
import { RootState } from '@store/index';
import { getFile } from '../tomb/actions';

export const loadFilePreview = createAsyncThunk(
    'loadFilePreview',
    async(_, { dispatch, getState }) => {
        const { filePreview: { file, bucket, path } } = getState() as RootState;
        const arrayBuffer = unwrapResult(await dispatch(getFile({ bucket: bucket!, path, name: file.name })));
        const blob = new File([arrayBuffer], file.name, { type: file.mimeType });

        return URL.createObjectURL(blob);
    }
);
