import React, { useMemo } from 'react';
import { useSearchParams } from 'next/navigation';
import Link from 'next/link';
import { useIntl } from 'react-intl';
import { IoMdAdd } from 'react-icons/io';

import { NextPageWithLayout } from '../page';
import { useTomb } from '@/contexts/tomb';

import BaseLayout from '@/layouts/BaseLayout';
import { BucketTable } from '@/components/Buckets/BucketTable';
import { Fallback } from '@/components/common/Fallback';

import getServerSideProps from '@/utils/session';

export { getServerSideProps };

const Bucket: NextPageWithLayout = () => {
    const searchParams = useSearchParams();
    const bucketId = searchParams.get('id');
    const { buckets, areBucketsLoading } = useTomb();
    const selectedBucket = useMemo(() => buckets.find(bucket => bucket.id === bucketId), [buckets, bucketId]);

    const uploadFile = (event: React.ChangeEvent<HTMLInputElement>) => { };
    const { messages } = useIntl();

    return (
        <section className="py-9 px-4">
            <div className="mb-4 flex w-full justify-between items-center">
                <h2 className="text-xl font-semibold">
                    <Link href="/">{`${messages.myBuckets}`}</Link>
                    {' > '}
                    <Link href={`/bucket/${bucketId}`}>{selectedBucket?.name}</Link>
                </h2>
                <label className="flex gap-2 w-40 items-center justify-center py-2 px-4 font-semibold cursor-pointer rounded-lg bg-blue-primary text-white">
                    <IoMdAdd fill="#fff" size="20px" />
                    {`${messages.upload}`}
                    <input
                        type="file"
                        className="hidden"
                        onChange={uploadFile}
                    />
                </label>
            </div>
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
