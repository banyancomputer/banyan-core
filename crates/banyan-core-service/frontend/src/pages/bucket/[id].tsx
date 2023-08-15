import React, { useMemo } from 'react';
import { useSearchParams } from 'next/navigation';
import { useIntl } from 'react-intl';
import { IoMdAdd } from "react-icons/io";

import { useTomb } from '@/contexts/tomb';

import BaseLayout from '@/layouts/BaseLayout';
import { NextPageWithLayout } from '../page';
import { BucketTable } from '@/components/Buckets/BucketsTable';

import { Upload } from '@static/images/buckets';

const Bucket: NextPageWithLayout = () => {
    const searchParams = useSearchParams();
    const bucketId = searchParams.get('id');
    const { buckets } = useTomb();
    const selectedBucket = useMemo(() => buckets.find(bucket => bucket.id === bucketId), [buckets, bucketId]);

    const uploadFile = (event: React.ChangeEvent<HTMLInputElement>) => { };
    const { messages } = useIntl();

    return (
        <section className="py-9 px-4">
            <div className="mb-4 flex w-full justify-between items-center">
                <h2 className="text-xl font-semibold">
                    {selectedBucket?.name}
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
            <BucketTable buckets={selectedBucket ? [selectedBucket] : []} />
            <label className="mt-10 flex flex-col items-center justify-center gap-4 px-6 py-4 border-2 border-c rounded-xl  text-xs cursor-pointer">
                <Upload />
                <span className="text-gray-600">
                    <b className="text-gray-900">{`${messages.clickToUpload}`} </b>
                    {`${messages.orDragAndDrop}`}
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
