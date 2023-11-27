import React from 'react';
import { Link, useParams } from 'react-router-dom';
import { useIntl } from 'react-intl';

import { TakeSnapshotModal } from '@components/common/Modal/TakeSnapshotModal';
import { UploadFileModal } from '@/app/components/common/Modal/UploadFileModal';

import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { stringToBase64 } from '@app/utils/base64';

import { Copy, PlusBold } from '@static/images/common';

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

    const takeSnapshot = async () => {
        try {
            selectedBucket && openModal(<TakeSnapshotModal bucket={selectedBucket} />);
        } catch (error: any) { }
    };

    return (
        <div className="mb-6">
            <div className="mb-4 flex w-full justify-between items-center">
                <h2 className="text-xl font-semibold">
                    <Link to="/">{`${messages.myDrives}`}</Link>
                    {' > '}
                    <Link to={`/drive/${bucketId}`}>{selectedBucket?.name}</Link>
                    {folderLocation.map((folder, index) =>
                        <React.Fragment key={index}>
                            {' > '}
                            <Link to={`/drive/${bucketId}?${folderLocation.slice(0, ++index).map(element => stringToBase64(element)).join('/')}`}>{folder}</Link>
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
            <div className="flex items-center justify-center gap-3 px-5 py-4 bg-secondaryBackground rounded-xl">
                <span className="p-2.5 rounded-full bg-button-primary">
                    <Copy />
                </span>
                <div className="flex-grow flex flex-col text-text-900">
                    <h6 className="font-semibold">{`${messages.archivalSnapshots}`}</h6>
                    <p>{`${selectedBucket?.isSnapshotValid ? messages.driveHasSnapshot : messages.driveHasNoSnapshot}`}</p>
                    <p className="underline cursor-pointer" title={`${messages.snapshotTooltip}`}>{`${messages.whatIsSnapshot}`}</p>
                </div>
                <button
                    onClick={takeSnapshot}
                    disabled={selectedBucket?.isSnapshotValid}
                    className='px-4 py-2.5 border-1 border-button-highLight rounded-xl disabled:border-text-900 disabled:opacity-20 disabled:cursor-not-allowed'
                >
                    {`${messages.makeSnapshot}`}
                </button>
            </div>
        </div>
    );
};

export default BucketHeader;
