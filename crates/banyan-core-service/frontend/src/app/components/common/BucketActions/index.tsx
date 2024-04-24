import React from 'react';
import { useNavigate } from 'react-router-dom';

import { RenameBucketModal } from '@components/common/Modal/RenameBucketModal';
import { DeleteDriveModal } from '@/app/components/common/Modal/DeleteDriveModal';
import { TakeSnapshotModal } from '@components/common/Modal/TakeSnapshotModal';
import { UploadFileModal } from '@components/common/Modal/UploadFileModal';
import { CreateFolderModal } from '@components/common/Modal/CreateFolderModal ';
import { Tooltip } from '@components/common/Tooltip';

import { Action } from '@components/Bucket/Files/BucketTable/FileActions';
import { closeModal, openModal } from '@store/modals/slice';
import { Bucket } from '@/app/types/bucket';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';
import { useTomb } from '@contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useAppDispatch, useAppSelector } from '@/app/store';

import { Bolt, DeleteHotData, Rename, Retry, Trash, Upload, Versions } from '@static/images/common';
import { AddFolderIcon, Lock } from '@static/images/buckets';

export const BucketActions: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.bucketActions);
    const { remountBucket } = useTomb();
    const bucketType = `${bucket.bucketType}_${bucket.storageClass}`;
    const folderLocation = useFolderLocation();
    const navigate = useNavigate();
    const dispatch = useAppDispatch();

    const hideModal = () => {
        dispatch(closeModal());
    }

    const upload = async () => {
        try {
            dispatch(openModal(
                {
                    content: <UploadFileModal
                        bucket={bucket}
                        path={folderLocation}
                    />
                }
            ));
        } catch (error: any) { }
    };

    const takeSnapshot = async () => {
        try {
            dispatch(openModal(
                {
                    content: <TakeSnapshotModal bucket={bucket} />
                }
            ));
        } catch (error: any) { }
    };

    const viewBucketSnapshots = async () => {
        try {
            navigate(`/drive/${bucket.id}/snapshots`);
        } catch (error: any) { }
    };

    const viewBucketVersions = async () => {
        try {
            // openModal(<BucketSnapshotsModal bucketId={bucket.id} />);
        } catch (error: any) { }
    };

    const rename = async () => {
        dispatch(openModal({
            content: <RenameBucketModal bucket={bucket} />
        }));
    };

    const createFolder = async () => {
        dispatch(openModal(
            {
                content: <CreateFolderModal bucket={bucket} path={folderLocation} onSuccess={hideModal} />
            }
        ));
    };

    const retoreColdVersion = async () => {
        try {

        } catch (error: any) { }
    };
    const deleteHotData = async () => {
        try {

        } catch (error: any) { }
    };

    const deleteBucket = async () => {
        try {
            dispatch(openModal(
            {
                content: <DeleteDriveModal bucket={bucket} />
            }
        ));
        } catch (error: any) { }
    };

    const purgeColdKeys = async () => {
        try {
        } catch (error: any) { }
    };

    const unlock = async () => {
        try {
            /** TODO: implement after will be added into tomb wasm. */
        } catch (error: any) { }
    };

    const remount = async () => {
        try {
            await remountBucket(bucket);
        } catch (error: any) {
            ToastNotifications.error('Error on bucket remount', 'Try again', remount);
        }
    }

    const uploadAction = new Action(`${messages.upload}`, <Upload width="18px" height="18px" />, upload);
    const createSnapshotAction = bucket.isSnapshotValid || !bucket.files.length ? null : new Action(`${messages.takeArchivalSnapshot}`, <Bolt width="18px" height="18px" />, takeSnapshot, `${messages.snapshotExplanation}`);
    const viewBucketSnapshotsAction = bucket.snapshots.length ? new Action(`${messages.viewArchivalSnapshots}`, <Versions width="18px" height="18px" />, viewBucketSnapshots) : null;
    const viewBucketVersionsAction = new Action(`${messages.viewDriveVersions}`, <Versions width="18px" height="18px" />, viewBucketVersions);
    const renameAction = new Action(`${messages.rename}`, <Rename width="18px" height="18px" />, rename);
    const createFolderAction = new Action(`${messages.createFolder}`, <AddFolderIcon width="18px" height="18px" />, createFolder);
    const restoreColdVersionAction = new Action(`${messages.restoreCold}`, <Versions width="18px" height="18px" />, retoreColdVersion);
    const deleteHotDatadAction = new Action(`${messages.deleteHotData}`, <DeleteHotData width="18px" height="18px" />, deleteHotData);
    const deletedAction = new Action(`${messages.delete}`, <Trash width="18px" height="18px" />, deleteBucket);
    const purgeAction = new Action(`${messages.purgeColdKeys}`, <Trash width="18px" height="18px" />, purgeColdKeys);

    const hotInrecactiveActions = [
        createFolderAction, uploadAction, createSnapshotAction, viewBucketSnapshotsAction, renameAction, deletedAction
    ];
    const warmInrecactiveActions = [
        createFolderAction, uploadAction, createSnapshotAction, restoreColdVersionAction, viewBucketVersionsAction, deleteHotDatadAction, purgeAction, deletedAction,
    ];
    const coldIntecactiveActions = [
        createFolderAction, viewBucketSnapshotsAction, renameAction, viewBucketVersionsAction, purgeAction, deletedAction,
    ];
    const hotBackupActions = [
        createSnapshotAction, renameAction, viewBucketSnapshotsAction, deletedAction,
    ];
    const warmBackupActions = [
        viewBucketSnapshotsAction, createSnapshotAction, restoreColdVersionAction, viewBucketVersionsAction, deleteHotDatadAction, purgeAction, deletedAction,
    ];
    const coldBackupActions = [
        viewBucketSnapshotsAction, restoreColdVersionAction, renameAction, purgeAction, deletedAction,
    ];

    const actions: Record<string, Array<Action | null>> = {
        interactive_hot: hotInrecactiveActions,
        interactive_warm: warmInrecactiveActions,
        interactive_cold: coldIntecactiveActions,
        backup_hot: hotBackupActions,
        backup_warm: warmBackupActions,
        backup_cold: coldBackupActions,
    };

    return (
        <div className={'w-64 text-xs font-medium bg-bucket-actionsBackground rounded-md overflow-hidden shadow-md z-10 select-none text-bucket-actionsText'}>
            {bucket.mount ?
                <>
                    {
                        bucket.locked ?
                            <>
                                <div
                                    className="w-full flex items-center gap-2 py-2 px-3 transition-colors hover:bg-hover"
                                    onClick={deletedAction.value}
                                >
                                    {deletedAction.icon}
                                    {deletedAction.label}
                                </div>
                                <div
                                    className="w-full flex items-center gap-2 py-2 px-3 transition-colors hover:bg-hover"
                                    onClick={unlock}
                                >
                                    <Lock width="18px" height="18px" color="currentColor" />
                                    {`${messages.unlock}`}
                                </div>
                            </>
                            :
                            actions[bucketType].map(action =>
                                action ?
                                    <div
                                        key={action.label}
                                        className="w-full flex items-center gap-2 py-2 px-3 transition-colors hover:bg-hover"
                                        onClick={action.value}
                                    >
                                        {action.icon}
                                        {action.label}
                                        {action.tooltip ?
                                            <Tooltip
                                                body={<span >(?)</span>}
                                                tooltip={<>{action.tooltip}</>}
                                                bodyClassName="right-10"
                                            />
                                            :
                                            null
                                        }
                                    </div>
                                    :
                                    null
                            )
                    }
                </>
                :
                <div
                    className="w-full flex items-center gap-2 py-2 px-3 transition-colors hover:bg-hover"
                    onClick={remount}
                >
                    <Retry width="18px" height="18px" />
                    {`${messages.unlock}`}
                </div>
            }
        </div>
    );
};
