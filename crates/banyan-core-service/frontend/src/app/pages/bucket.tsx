import React, { useEffect } from 'react';
import { Outlet, useParams } from 'react-router-dom';
import { unwrapResult } from '@reduxjs/toolkit';

import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { useAppDispatch, useAppSelector } from '@store/index';
import { getSelectedBucketFiles, mountBucket } from '@store/tomb/actions';
import { selectBucket, setIsLoading } from '@store/tomb/slice';

const Bucket = () => {
    const params = useParams();
    const dispatch = useAppDispatch();
    const { selectedBucket, buckets } = useAppSelector(state => state.tomb);
    const bucketId = params.id;
    const folderLocation = useFolderLocation();

    useEffect(() => {
        if (selectedBucket?.id !== bucketId || selectedBucket?.locked || !selectedBucket?.mount) { return; }
        (async () => {
            try {
                unwrapResult(await dispatch(getSelectedBucketFiles(folderLocation)));
            } catch (error: any) { };
        })();
    }, [folderLocation, selectedBucket?.id, selectedBucket?.mount]);

    useEffect(() => {
        const bucket = buckets.find(bucket => bucket.id === bucketId);
        bucket && dispatch(selectBucket(bucket));
    }, [bucketId, buckets]);

    useEffect(() => () => {
        dispatch(selectBucket(null));
    }, []);

    useEffect(() => {
        const bucket = buckets.find(bucket => bucket.id === bucketId);

        if (bucket?.mount) return;

        (async () => {
            try {
                dispatch(setIsLoading(true));
                unwrapResult(await dispatch(mountBucket(selectedBucket!)));
                dispatch(setIsLoading(false));
            } catch (error: any) {
                console.log(error);
            }
        })()
    }, [selectedBucket?.mount, selectedBucket?.id, buckets])

    return (
        <section className="flex flex-col flex-grow">
            <Outlet />
        </section>
    );
};

export default Bucket;
