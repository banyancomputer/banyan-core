import React, { useEffect } from 'react';
import { useIntl } from 'react-intl';
import Image from 'next/image';

import BaseLayout from '@layouts/BaseLayout';
import { NextPageWithLayout } from './page';

import { UploadFileModal } from '@/components/common/Modal/UploadFileModal';
import { Fallback } from '@/components/common/Fallback';
import { Bucket } from '@/components/Buckets/Bucket';

import getServerSideProps from '@/utils/session';
import { useTomb } from '@/contexts/tomb';
import { useModal } from '@/contexts/modals';

import emptyIcon from '@static/images/common/emptyIcon.png';
import { PlusBold } from '@static/images/common';

export { getServerSideProps };

const Buckets: NextPageWithLayout = () => {
    const { openModal } = useModal();
    const { buckets, areBucketsLoading, getBucketsFiles, tomb } = useTomb();
    const { messages } = useIntl();

    const uploadFile = () => {
        openModal(<UploadFileModal />);
    };

    useEffect(() => {
        if (!tomb) return;

        (async () => {
            await getBucketsFiles();
        })();
    }, [buckets.length, tomb]);

    return (
        <section className="py-9 px-4" id="buckets">
            <div className="mb-4 flex w-full justify-between items-center">
                <h2 className="text-xl font-semibold">
                    {`${messages.myBuckets}`}
                </h2>
                <button
                    className="btn-highlighted gap-2 w-40 py-2 px-4"
                    onClick={uploadFile}
                >
                    <PlusBold />
                    {`${messages.upload}`}
                </button>
            </div>
            <Fallback shouldRender={!areBucketsLoading}>
                {buckets.length ?
                    <div className='grid grid-cols-3 gap-3'>{
                        buckets.map(bucket =>
                            <Bucket bucket={bucket} key={bucket.id} />
                        )
                    }</div>
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
