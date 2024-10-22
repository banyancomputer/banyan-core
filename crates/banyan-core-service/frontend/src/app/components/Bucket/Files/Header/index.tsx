import React, { useEffect, useState } from 'react';
import { Link, useParams } from 'react-router-dom';

import { TakeSnapshotModal } from '@components/common/Modal/TakeSnapshotModal';
import { UploadFileModal } from '@components/common/Modal/UploadFileModal';
import { CreateFolderModal } from '@components/common/Modal/CreateFolderModal ';

import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { closeModal, openModal } from '@store/modals/slice';
import { stringToBase64 } from '@utils/base64';
import { getLocalStorageItem, setLocalStorageItem } from '@utils/localStorage';
import { useAppDispatch, useAppSelector } from '@store/index';
import { Tooltip } from '@components/common/Tooltip';

import { Close, Copy, Upload } from '@static/images/common';
import { AddFolderIcon } from '@static/images/buckets';
import { StorageUsageClient } from '@/api/storageUsage';
import { convertFileSize } from '@/app/utils/storage';

const storageUsageClient = new StorageUsageClient();

const BucketHeader = () => {
    const messages = useAppSelector(state => state.locales.messages.coponents.bucket.files.header);
    const folderLocation = useFolderLocation();
    const { selectedBucket, isLoading } = useAppSelector(state => state.tomb);
    const params = useParams();
    const bucketId = params.id;
    const [isBannerVisible, setIsBannerVisible] = useState(false);
    const [storageUsage, setStorageUsage] = useState(0);
    const dispatch = useAppDispatch();

    const uploadFile = () => {
        if (selectedBucket) {
            dispatch(openModal(
                {
                    content: <UploadFileModal
                        bucket={selectedBucket}
                        path={folderLocation}
                    />,
                    path: [selectedBucket.name, ...folderLocation]
                }
            )
            );
        }
    };

    const takeSnapshot = async () => {
        try {
            selectedBucket && dispatch(openModal({
                content: <TakeSnapshotModal bucket={selectedBucket} />
            }));
        } catch (error: any) { }
    };

    const closeBanner = () => {
        setIsBannerVisible(false);
        setLocalStorageItem('has_dissmissed_snapshot_banner', 'true');
    };

    const hideModal = () => {
        dispatch(closeModal());
    };

    const createFolder = () => {
        dispatch(openModal(
            {
                content: <CreateFolderModal
                    bucket={selectedBucket!}
                    path={folderLocation}
                    onSuccess={hideModal}
                />,
                path: [selectedBucket?.name || '', ...folderLocation]
            }
        )
        );
    };

    useEffect(() => {
        const hasUserDissmissedBanner = getLocalStorageItem('has_dissmissed_snapshot_banner');
        if (hasUserDissmissedBanner) { return; }

        if (selectedBucket?.files.length && !selectedBucket.isSnapshotValid) {
            setIsBannerVisible(true);
        };

        if (selectedBucket?.isSnapshotValid) {
            setIsBannerVisible(false);
        };
    }, [selectedBucket?.files, selectedBucket?.isSnapshotValid]);

    useEffect(() => {
        if (!selectedBucket?.id) return;

        (async () => {
            const storageUsage = await storageUsageClient.getStorageUsageForBucket(selectedBucket?.id);
            setStorageUsage(storageUsage)
        })();
    }, [selectedBucket?.id]);

    return (
        <div className="mb-8">
            <div className="mb-4 flex flex-col w-full">
                {!isLoading
                    ?
                    <>
                        <h2 className="mb-2 text-lg font-semibold">
                            <Link to={`/drive/${bucketId}`}>{selectedBucket?.name}</Link>
                            {folderLocation.map((folder, index) =>
                                <React.Fragment key={index}>
                                    {' > '}
                                    <Link to={`/drive/${bucketId}?${folderLocation.slice(0, ++index).map(element => stringToBase64(element)).join('/')}`}>{folder}</Link>
                                </React.Fragment>
                            )}
                        </h2>
                        <div className="mb-4 flex items-center gap-2 text-text-400 text-xs">
                            {selectedBucket?.files.length} {messages.files}
                            <span className="w-1 h-1 bg-text-400 rounded-full" />
                            {convertFileSize(storageUsage)}
                        </div>
                        {selectedBucket?.bucketType !== 'backup' && !selectedBucket?.locked &&
                            <div className="flex items-stretch gap-2">
                                <button
                                    className="btn-primary gap-2 w-40 py-2 px-4 bg-"
                                    onClick={uploadFile}
                                >
                                    <Upload />
                                    {messages.uploadButton}
                                </button>
                                <button
                                    className="flex items-center gap-2 py-2 px-4 border-1 border-border-regular rounded-md text-text-900 font-semibold"
                                    onClick={createFolder}
                                >
                                    <AddFolderIcon width="20px" height="20px" />
                                    {messages.createFolderButton}
                                </button>
                            </div>
                        }
                    </>
                    :
                    null
                }
            </div>
            {isBannerVisible &&
                <div className="relative flex items-center justify-center gap-3 px-5 py-4 border-2 border-border-regular rounded-xl">
                    <span className="p-2.5 rounded-full bg-button-primary text-white">
                        <Copy />
                    </span>
                    <div className="flex-grow flex flex-col text-text-900">
                        <h6 className="font-semibold">{messages.snapshotBannerTitle}</h6>
                        <p>{messages.snapshotBannerSubtitle}</p>
                        <Tooltip
                            body={<p className="underline cursor-pointer">{messages.snapshotBannerExplanation}</p>}
                            tooltip={<div className="p-2 bg-bucket-actionsBackground rounded-md">{`${messages.snapshotBannerTooltip}`}</div>}
                        />
                    </div>
                    <button
                        onClick={takeSnapshot}
                        disabled={selectedBucket?.isSnapshotValid}
                        className="px-4 py-2.5 btn-primary rounded-xl"
                    >
                        {messages.makeSnapshot}
                    </button>
                    <button onClick={closeBanner}>
                        <Close />
                    </button>
                </div>
            }
        </div>
    );
};

export default BucketHeader;
