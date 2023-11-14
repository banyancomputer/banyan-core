import React from 'react';
import { Link, useParams } from 'react-router-dom';
import { useIntl } from 'react-intl';

import { UploadFileModal } from '@/app/components/common/Modal/UploadFileModal';

import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { stringToBase64 } from '@app/utils/base64';

import { PlusBold } from '@static/images/common';

const BucketHeader = () => {
    const { messages } = useIntl();
    const folderLocation = useFolderLocation();
    const { selectedBucket } = useTomb();
    const params = useParams();
    const bucketId = params.id;
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
                <Link to="/">{`${messages.myDrives}`}</Link>
                {' > '}
                <Link to={`/bucket/${bucketId}`}>{selectedBucket?.name}</Link>
                {folderLocation.map((folder, index) =>
                    <React.Fragment key={index}>
                        {' > '}
                        <Link to={`/bucket/${bucketId}?${folderLocation.slice(0, ++index).map(element => stringToBase64(element)).join('/')}`}>{folder}</Link>
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

export default BucketHeader;
