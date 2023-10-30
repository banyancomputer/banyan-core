import React, { useEffect } from 'react';
import { useRouter } from 'next/router';

import { NextPageWithLayout } from '../page';
import { useTomb } from '@/contexts/tomb';
import getServerSideProps from '@/utils/session';

import BaseLayout from '@/layouts/BaseLayout';
import { BucketTable } from '@/components/Bucket/BucketTable';
import { Fallback } from '@/components/common/Fallback';
import { useFolderLocation } from '@/hooks/useFolderLocation';
import BucketHeader from '@/components/Bucket/Header';

export { getServerSideProps };

const Bucket: NextPageWithLayout = () => {
    const router = useRouter();
    const bucketId = router.query.id;
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

Bucket.getLayout = (page) => <BaseLayout>{page}</BaseLayout>;
