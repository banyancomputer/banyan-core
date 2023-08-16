import React, { useEffect, useState } from 'react';
import { useIntl } from 'react-intl';
import { IoMdAdd } from "react-icons/io"

import BaseLayout from '@layouts/BaseLayout';
import { NextPageWithLayout } from './page';
import { BucketsTable } from '@/components/Buckets/BucketsTable';
import { UploadFileModal } from '@/components/common/Modal/UploadFileModal';

import { useTomb } from '@/contexts/tomb';
import { useModal } from '@/contexts/modals';

import { Upload } from '@static/images/buckets';

const Buckets: NextPageWithLayout = () => {
    const { openModal } = useModal();
    const { buckets } = useTomb();
    const { messages } = useIntl();

    const uploadFile = () => {
        openModal(<UploadFileModal />)
    };

    return (
        <section className="py-9 px-4" id='buckets'>
            <div className="mb-4 flex w-full justify-between items-center">
                <h2 className="text-xl font-semibold">
                    {`${messages.myBuckets}`}
                </h2>
                <button
                    className="btn-primary gap-2 w-40 py-2 px-4"
                    onClick={uploadFile}
                >
                    <IoMdAdd fill="#fff" size="20px" />
                    {`${messages.upload}`}
                </button>
            </div>
            <BucketsTable buckets={buckets} />
            <div
                className="mt-10 flex flex-col items-center justify-center gap-4 px-6 py-4 border-2 border-c rounded-xl  text-xs cursor-pointer"
                onClick={uploadFile}
            >
                <Upload />
                <span className="text-gray-600">
                    <b className="text-gray-900">{`${messages.clickToUpload}`} </b>
                    {`${messages.orDragAndDrop}`}
                </span>
            </div>
        </section>
    );
};


export default Buckets;

Buckets.getLayout = (page) => <BaseLayout>{page}</BaseLayout>;
