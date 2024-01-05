import React, { useEffect } from 'react';
import { useParams } from 'react-router-dom';

import { BucketTable } from '@components/Bucket/BucketTable';
import { Fallback } from '@components/common/Fallback';
import BucketHeader from '@components/Bucket/Header';

import { useTomb } from '@/app/contexts/tomb';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { EmptyState } from '@components/Bucket/EmptyState';

const Bucket = () => {
    const params = useParams();
    const bucketId = params.id;
    const { buckets, areBucketsLoading, selectedBucket, selectBucket, getSelectedBucketFiles } = useTomb();
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
        <section className="py-9 px-10 flex flex-col flex-grow max-h-[calc(100vh-132px)]">
            <BucketHeader />
            <Fallback shouldRender={!areBucketsLoading}>
                {selectedBucket &&
                    <>
                        {selectedBucket.files.length ?
                            <BucketTable bucket={selectedBucket} />
                            :
                            <EmptyState bucket={selectedBucket} />
                        }
                    </>
                }
            </Fallback>
        </section>
    );
};

export default Bucket;
