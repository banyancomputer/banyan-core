import React, { useEffect } from 'react';
import { Outlet, useParams } from 'react-router-dom';

import { useTomb } from '@/app/contexts/tomb';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';

const Bucket = () => {
    const params = useParams();
    const bucketId = params.id;
    const { buckets, selectedBucket, selectBucket, getSelectedBucketFiles } = useTomb();
    const folderLocation = useFolderLocation();

    useEffect(() => {
        if (selectedBucket?.id !== bucketId || selectedBucket?.locked) { return; }
        (async () => {
            try {
                getSelectedBucketFiles(folderLocation);
            } catch (error: any) { };
        })();
    }, [folderLocation, selectedBucket?.id]);

    useEffect(() => {
        const bucket = buckets.find(bucket => bucket.id === bucketId);
        bucket && selectBucket(bucket);
    }, [bucketId, buckets.length]);

    useEffect(() => () => {
        selectBucket(null);
    }, []);

    return (
        <section className="flex flex-col flex-grow">
            <Outlet />
        </section>
    );
};

export default Bucket;
