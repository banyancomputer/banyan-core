import React, { useEffect, useState } from 'react';

import BaseLayout from '@layouts/BaseLayout';
import { NextPageWithLayout } from './page';
import { BucketTable } from '@/components/Buckets/BucketsTable';

import { useTomb } from '@/contexts/tomb';
import { useModal } from '@/contexts/modals';

import { Add, Upload } from '@static/images/buckets';
import { UploadFileModal } from '@/components/common/Modal/UploadFileModal';

const Buckets: NextPageWithLayout = () => {
    const { openModal } = useModal()
    const { buckets } = useTomb();

    const uploadFile = () => {
        openModal(<UploadFileModal />)
    };

    return (
        <section className="py-9 px-4">
            <div className="mb-4 flex w-full justify-between items-center">
                <h2 className="text-xl font-semibold">
                    My Buckets
                </h2>
                <button
                    className="flex gap-2 w-40 items-center justify-center py-2 px-4 font-semibold cursor-pointer rounded-lg bg-blue-primary text-white"
                    onClick={uploadFile}
                >
                    <Add />
                    Upload
                </button>
            </div>
            <BucketTable buckets={buckets} />
            <div
                className="mt-10 flex flex-col items-center justify-center gap-4 px-6 py-4 border-2 border-c rounded-xl  text-xs cursor-pointer"
                onClick={uploadFile}
            >
                <Upload />
                <span className="text-gray-600">
                    <span className="font-semibold text-black">Click to upload </span>
                    or drag and drop
                </span>
            </div>
        </section>
    );
};


export default Buckets;

Buckets.getLayout = (page) => <BaseLayout>{page}</BaseLayout>;
