import React, { useEffect } from 'react';
import { Outlet, useParams } from 'react-router-dom';
import { unwrapResult } from '@reduxjs/toolkit';

import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { useAppDispatch, useAppSelector } from '@store/index';
import { getSelectedBucketFiles, mountBucket } from '@store/tomb/actions';
import { selectBucket } from '@store/tomb/slice';

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
    }, [bucketId, buckets.length]);

    useEffect(() => () => {
        dispatch(selectBucket(null));
    }, []);

    useEffect(() => {
        if (selectedBucket?.mount) return;

        (async () => {
            try {
                unwrapResult(await dispatch(mountBucket(selectedBucket!.id)));
            } catch (error: any) {
                console.log(error);
            }
        })()
    }, [selectedBucket?.mount, selectedBucket?.id])

    return (
        <section className="flex flex-col flex-grow">
            <Outlet />
        </section>
    );
};

export default Bucket;
