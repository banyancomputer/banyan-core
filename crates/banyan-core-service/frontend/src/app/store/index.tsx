import { configureStore } from '@reduxjs/toolkit';
import { TypedUseSelectorHook, useDispatch, useSelector } from 'react-redux';

import session from '@store/session/slice';
import keystore from '@store/keystore/slice';
import billing from '@store/billing/slice';
import locales from '@store/locales/slice';
import errors from '@store/errors/slice';
import modals from '@store/modals/slice';
import tomb from '@store/tomb/slice';
import filePreview from '@store/filePreview/slice';
import filesUpload from '@store/filesUpload/slice';

export const store = configureStore({
    reducer: {
        errors,
        billing,
        locales,
        session,
        keystore,
        modals,
        filePreview,
        filesUpload,
        tomb,
    },
});

// Infer the `RootState` and `AppDispatch` types from the store itself
export type RootState = ReturnType<typeof store.getState>;
// Inferred type: {posts: PostsState, comments: CommentsState, users: UsersState}
export type AppDispatch = typeof store.dispatch;

/** Hook dispatch for redux toolkit */
export const useAppDispatch = () => useDispatch<AppDispatch>();

/** Hook useSelector hook for redux toolkit */
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector;
