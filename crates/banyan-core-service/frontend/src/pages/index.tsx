import React, { useEffect } from 'react';
import { useIntl } from 'react-intl';
import { IoMdAdd } from 'react-icons/io';
import Image from 'next/image';

import BaseLayout from '@layouts/BaseLayout';
import { NextPageWithLayout } from './page';
import { BucketsTable } from '@/components/Buckets/BucketsTable';
import { UploadFileModal } from '@/components/common/Modal/UploadFileModal';
import { Fallback } from '@/components/common/Fallback';

import getServerSideProps from '@/utils/session';
import { useTomb } from '@/contexts/tomb';
import { useModal } from '@/contexts/modals';

import emptyIcon from '@static/images/common/emptyIcon.png';

export { getServerSideProps };

const Buckets: NextPageWithLayout = () => {
    const { openModal } = useModal();
    const { buckets, areBucketsLoading } = useTomb();
    const { messages } = useIntl();

    const uploadFile = () => {
        openModal(<UploadFileModal />);
    };

    return (
        <section className="py-9 px-4" id="buckets">
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
            <Fallback shouldRender={!areBucketsLoading}>
                {buckets.length ?
                    <BucketsTable buckets={buckets} />
                    :
                    <div className="h-full flex flex-col items-center justify-center saturate-0">
                        <Image src={emptyIcon} alt="emptyIcon" />
                        <p className="mt-4">{`${messages.noBuckets}`}</p>
                    </div>
                }
            </Fallback>
        </section>
    );
};


export default Buckets;

Buckets.getLayout = (page) => <BaseLayout>{page}</BaseLayout>;
