import React from 'react';
import { useSearchParams } from 'next/navigation';
import Link from 'next/link';
import dynamic from 'next/dynamic';
import { useIntl } from 'react-intl';
import { IoMdAdd } from 'react-icons/io';

import { UploadFileModal } from '@/components/common/Modal/UploadFileModal';

import { useFolderLocation } from '@/hooks/useFolderLocation';
import { useModal } from '@/contexts/modals';
import { useTomb } from '@/contexts/tomb';

const BucketHeader = () => {
    const searchParams = useSearchParams();
    const { messages } = useIntl();
    const folderLocation = useFolderLocation();
    const { selectedBucket } = useTomb();
    const bucketId = searchParams.get('id');
    const { openModal } = useModal();

    const uploadFile = () => {
        if (selectedBucket) {
            openModal(<UploadFileModal bucket={selectedBucket} />);
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
                    className="btn-primary gap-2 w-40 py-2 px-4"
                    onClick={uploadFile}
                >
                    <IoMdAdd fill="#fff" size="20px" />
                    {`${messages.upload}`}
                </button>
            }
        </div>
    );
};

export default dynamic(() => Promise.resolve(BucketHeader), {
    ssr: false,
});
