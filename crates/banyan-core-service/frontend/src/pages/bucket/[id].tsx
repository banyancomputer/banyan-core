import React, { useMemo } from 'react';
import { useSearchParams } from 'next/navigation';
import Link from 'next/link';
import { useIntl } from 'react-intl';
import { IoMdAdd } from 'react-icons/io';
import Image from 'next/image';

import { NextPageWithLayout } from '../page';
import { useTomb } from '@/contexts/tomb';
import { useModal } from '@/contexts/modals';
import getServerSideProps from '@/utils/session';

import BaseLayout from '@/layouts/BaseLayout';
import { BucketTable } from '@/components/Buckets/BucketTable';
import { Fallback } from '@/components/common/Fallback';
import { UploadFileModal } from '@/components/common/Modal/UploadFileModal';

export { getServerSideProps };

const Bucket: NextPageWithLayout = () => {
    const searchParams = useSearchParams();
    const bucketId = searchParams.get('id');
    const { buckets, areBucketsLoading } = useTomb();
    const { messages } = useIntl();
    const { openModal, closeModal } = useModal();
    const selectedBucket = useMemo(() => buckets.find(bucket => bucket.id === bucketId), [buckets, bucketId]);

    const uploadFile = () => {
        openModal(<UploadFileModal bucket={selectedBucket} />)
    };

    return (
        <section className="py-9 px-4">
            <div className="mb-4 flex w-full justify-between items-center">
                <h2 className="text-xl font-semibold">
                    <Link href="/">{`${messages.myBuckets}`}</Link>
                    {' > '}
                    <Link href={`/bucket/${bucketId}`}>{selectedBucket?.name}</Link>
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
                {selectedBucket &&
                    <BucketTable bucket={selectedBucket} />
                }
            </Fallback>
        </section>
    );
};

export default Bucket;

Bucket.getLayout = (page) => <BaseLayout>{page}</BaseLayout>;
