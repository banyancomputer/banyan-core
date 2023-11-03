import React, { useEffect } from 'react';
// import { useRouter } from 'next/router';

import { useTomb } from '@/app/contexts/tomb';

import { BucketTable } from '@/app/components/Bucket/BucketTable';
import { Fallback } from '@/app/components/common/Fallback';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import BucketHeader from '@/app/components/Bucket/Header';
import { useParams } from 'react-router-dom';

const Bucket = () => {
    const params = useParams();
    const bucketId = params.id;

    const { buckets, areBucketsLoading, selectedBucket, selectBucket, getSelectedBucketFiles } = useTomb();
    const folderLocation = useFolderLocation();

    useEffect(() => {
        if (selectedBucket?.id !== bucketId) { return; }
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

    useEffect(() => {
        return () => {
            selectBucket(null);
        };
    }, []);

    return (
        <section className="py-9 px-4">
            <BucketHeader />
            <Fallback shouldRender={!areBucketsLoading}>
                {selectedBucket &&
                    <BucketTable bucket={selectedBucket} />
                }
            </Fallback>
        </section>
    );
};

export default Bucket;
