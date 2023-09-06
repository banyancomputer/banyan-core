import React, { useEffect, useMemo, useState } from 'react';
import { useSearchParams } from 'next/navigation';

import { NextPageWithLayout } from '../page';
import { useTomb } from '@/contexts/tomb';
import getServerSideProps from '@/utils/session';

import BaseLayout from '@/layouts/BaseLayout';
import { BucketTable } from '@/components/Buckets/BucketTable';
import { Fallback } from '@/components/common/Fallback';
import { useFolderLocation } from '@/hooks/useFolderLocation';
import { Bucket } from '@/lib/interfaces/bucket';
import BucketHeader from '@/components/Bucket/Header';


export { getServerSideProps };

const Bucket: NextPageWithLayout = () => {
    const searchParams = useSearchParams();
    const bucketId = searchParams.get('id');
    const { buckets, areBucketsLoading } = useTomb();
    const [selectedBucket, setSelectedBucket] = useState<Bucket>(buckets.find(bucket => bucket.id === bucketId)!);
    const folderLocation = useFolderLocation();

    useEffect(() => {
        if (!selectedBucket) return;
        (async () => {
            try {
                const files = await selectedBucket?.mount.ls(folderLocation);
                setSelectedBucket(bucket => ({ ...bucket, files }));
            } catch (error: any) { };
        })()
    }, [folderLocation])

    useEffect(() => {
        const bucket = buckets.find(bucket => bucket.id === bucketId)
        bucket && setSelectedBucket({ ...bucket, files: [...bucket.files] });
    }, [bucketId, buckets.length])

    return (
        <section className="py-9 px-4">
            <BucketHeader selectedBucket={selectedBucket} />
            <Fallback shouldRender={!areBucketsLoading}>
                {selectedBucket &&
                    <BucketTable bucket={selectedBucket} />
                }
            </Fallback>
        </section>
    );
};

export default Bucket

Bucket.getLayout = (page) => <BaseLayout>{page}</BaseLayout>;
