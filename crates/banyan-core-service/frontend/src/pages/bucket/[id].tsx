import React, { useMemo } from 'react';
import { useSearchParams } from 'next/navigation';
import { useTomb } from '@/contexts/tomb';

import BaseLayout from '@/layouts/BaseLayout';
import { NextPageWithLayout } from '../page';
import { BucketTable } from '@/components/Buckets/BucketsTable';

import { Add, Upload } from '@static/images/buckets';

const Bucket: NextPageWithLayout = () => {
    const searchParams = useSearchParams();
    const bucketId = searchParams.get('id');
    const { buckets } = useTomb();
    const selectedBucket = useMemo(() => buckets.find(bucket => bucket.id === bucketId), [buckets, bucketId]);

    const uploadFile = (event: React.ChangeEvent<HTMLInputElement>) => { };

    return (
        <section className="py-9 px-4">
            <div className="mb-4 flex w-full justify-between items-center">
                <h2 className="text-xl font-semibold">
                    {selectedBucket?.name}
                </h2>
                <label className="flex gap-2 w-40 items-center justify-center py-2 px-4 font-semibold cursor-pointer rounded-lg bg-blue-primary text-white">
                    <Add />
          Upload
                    <input
                        type="file"
                        className="hidden"
                        onChange={uploadFile}
                    />
                </label>
            </div>
            <BucketTable buckets={selectedBucket ? [selectedBucket] : []} />
            <label className="mt-10 flex flex-col items-center justify-center gap-4 px-6 py-4 border-2 border-c rounded-xl  text-xs cursor-pointer">
                <Upload />
                <span className="text-gray-600">
                    <span className="font-semibold text-black">Click to upload </span>
          or drag and drop
                </span>
                <input
                    type="file"
                    className="hidden"
                    onChange={uploadFile}
                />
            </label>
        </section>
    );
};

export default Bucket;

Bucket.getLayout = (page) => <BaseLayout>{page}</BaseLayout>;
