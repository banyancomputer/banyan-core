import React from 'react';
import { useIntl } from 'react-intl';

import { BucketSnapshotsModal } from '@/app/components/common/Modal/BucketSnapshotsModal';
import { RenameBucketModal } from '@/app/components/common/Modal/RenameBucketModal';
import { DeleteBucketModal } from '@/app/components/common/Modal/DeleteBucketModal';
import { TakeSnapshotModal } from '@/app/components/common/Modal/TakeSnapshotModal';

import { Action } from '../FileActions';
import { UploadFileModal } from '../Modal/UploadFileModal';
import { CreateFolderModal } from '../Modal/CreateFolderModal ';
import { useModal } from '@/app/contexts/modals';
import { Bucket } from '@/app/types/bucket';
import { useFolderLocation } from '@/app/hooks/useFolderLocation';


import { Bolt, DeleteHotData, Rename, Trash, Upload, Versions } from '@static/images/common';
import { Folder } from '@static/images/buckets';

export const BucketActions: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const { messages } = useIntl();
    const { openModal, closeModal } = useModal();
    const bucketType = `${bucket.bucketType}_${bucket.storageClass}`;
    const folderLocation = useFolderLocation();

    const upload = async () => {
        try {
            openModal(<UploadFileModal
                bucket={bucket}
                path={folderLocation}
            />
            );
        } catch (error: any) { }
    };

    const takeSnapshot = async () => {
        try {
            openModal(<TakeSnapshotModal bucket={bucket} />);
        } catch (error: any) { }
    };

    const viewBucketSnapshots = async () => {
        try {
            openModal(<BucketSnapshotsModal bucketId={bucket.id} />);
        } catch (error: any) { }
    };

    const viewBucketVersions = async () => {
        try {
            // openModal(<BucketSnapshotsModal bucketId={bucket.id} />);
        } catch (error: any) { }
    };

    const rename = async () => {
        openModal(<RenameBucketModal bucket={bucket} />);
    };

    const createFolder = async () => {
        openModal(<CreateFolderModal bucket={bucket} path={folderLocation} onSuccess={closeModal} />);
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
            openModal(<DeleteBucketModal bucket={bucket} />);
        } catch (error: any) { }
    };

    const purgeColdKeys = async () => {
        try {
        } catch (error: any) { }
    };

    const uploadAction = new Action(`${messages.upload}`, <Upload width="18px" height="18px" />, upload);
    const createSnapshotAction = new Action(`${messages.takeColdSnapshot}`, <Bolt width="18px" height="18px" />, takeSnapshot, `${messages.snapshotTooltip}`);
    const viewBucketSnapshotsAction = bucket.snapshots.length ? new Action(`${messages.viewColdSnapshots}`, <Versions width="18px" height="18px" />, viewBucketSnapshots) : null;
    const viewBucketVersionsAction = new Action(`${messages.viewDriveVersions}`, <Versions width="18px" height="18px" />, viewBucketVersions);
    const renameAction = new Action(`${messages.rename}`, <Rename width="18px" height="18px" />, rename);
    const createFolderAction = new Action(`${messages.createNewFolder}`, <Folder width="18px" height="18px" />, createFolder);
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
        <div className={'w-56 text-xs font-medium bg-bucket-actionsBackground rounded-xl overflow-hidden shadow-md z-10 select-none text-bucket-actionsText'}>
            {
                actions[bucketType].map(action =>
                    action ?
                        <div
                            key={action.label}
                            className="w-full flex items-center gap-2 py-2 px-3 transition-all hover:bg-hover"
                            onClick={action.value}
                        >
                            <span className="text-button-primary">
                                {action.icon}
                            </span>
                            {action.label}
                            {action.tooltip ?
                                <span className="text-button-primary" title={action.tooltip}>(?)</span>
                                :
                                null
                            }
                        </div>
                        :
                        null
                )
            }
        </div>
    );
};
