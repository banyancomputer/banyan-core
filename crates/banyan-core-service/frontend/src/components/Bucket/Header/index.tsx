import React from 'react';
import Link from 'next/link';
import dynamic from 'next/dynamic';
import { useRouter } from 'next/router';
import { useIntl } from 'react-intl';

import { UploadFileModal } from '@/components/common/Modal/UploadFileModal';

import { useFolderLocation } from '@/hooks/useFolderLocation';
import { useModal } from '@/contexts/modals';
import { useTomb } from '@/contexts/tomb';

import { PlusBold } from '@static/images/common';

const BucketHeader = () => {
    const { messages } = useIntl();
    const folderLocation = useFolderLocation();
    const { selectedBucket } = useTomb();
    const router = useRouter();
    const bucketId = router.query.id;
    const { openModal } = useModal();

    const uploadFile = () => {
        if (selectedBucket) {
            openModal(<UploadFileModal
                bucket={selectedBucket}
                path={folderLocation}
            />);
        }
    };

    return (
        <div className="mb-4 flex w-full justify-between items-center">
            <h2 className="text-xl font-semibold">
                <Link href="/">{`${messages.myBuckets}`}</Link>
                {' > '}
                <Link href={`/bucket/${bucketId}`}>{selectedBucket?.name}</Link>
                {folderLocation.map((folder, index) =>
                    <React.Fragment key={index}>
                        {' > '}
                        <Link href={`/bucket/${bucketId}?${folderLocation.slice(0, ++index).join('/')}`}>{folder}</Link>
                    </React.Fragment>
                )}
            </h2>
            {selectedBucket?.bucketType !== 'backup' &&
                <button
                    className="btn-highlighted bg-button-highLight gap-2 w-40 py-2 px-4 bg-"
                    onClick={uploadFile}
                >
                    <PlusBold />
                    {`${messages.upload}`}
                </button>
            }
        </div>
    );
};

export default dynamic(() => Promise.resolve(BucketHeader), {
    ssr: false,
});
