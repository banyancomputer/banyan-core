import { configureStore } from '@reduxjs/toolkit';
import { TypedUseSelectorHook, useDispatch, useSelector } from 'react-redux';

import user from '@app/store/user/slice';
import billing from '@app/store/billing/slice';

export const store = configureStore({
    reducer: {
        user,
        billing
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
