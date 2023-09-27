import React, { useMemo } from 'react';
import { useIntl } from 'react-intl';
import { FiEdit, FiTrash2, FiUpload } from 'react-icons/fi';
import { HiOutlineLightningBolt } from 'react-icons/hi';
import { MdRestore, MdOutlineRestoreFromTrash } from 'react-icons/md';
import { BsBoxSeam } from 'react-icons/bs';
import { PiFolderNotchPlusBold } from 'react-icons/pi';
import { useRouter } from 'next/router';

import { useModal } from '@/contexts/modals';
import { Bucket } from '@/lib/interfaces/bucket';
import { useFolderLocation } from '@/hooks/useFolderLocation';
import { useTomb } from '@/contexts/tomb';

import { Action } from '../FileActions';
import { BucketSnapshotsModal } from '@/components/common/Modal/BucketSnapshotsModal';
import { RenameBucketModal } from '@/components/common/Modal/RenameBucketModal';
import { DeleteBucketModal } from '@/components/common/Modal/DeleteBucketModal';
import { TakeSnapshotModal } from '@/components/common/Modal/TakeSnapshotModal';
import { UploadFileModal } from '../Modal/UploadFileModal';
import { CreateFolderModal } from '../Modal/CreateFolderModal ';

export const BucketActions: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const { messages } = useIntl();
    const { openModal, closeModal } = useModal();
    const bucketType = `${bucket.bucketType}_${bucket.storageClass}`;
    const { selectedBucket, getSelectedBucketFiles } = useTomb();
    const folderLocation = useFolderLocation();
    const router = useRouter();

    const upload = async () => {
        try {
            openModal(<UploadFileModal bucket={bucket} />);
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
        const onSuccess = async () => {
            await getSelectedBucketFiles(folderLocation);
            closeModal();
        };

        openModal(<CreateFolderModal bucket={bucket} path={folderLocation} onSuccess={onSuccess} />);
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

    const uploadAction = useMemo(() => new Action(`${messages.upload}`, <FiUpload size="18px" />, upload), []);
    const createSnapshotAction = useMemo(() => new Action(`${messages.takeColdSnapshot}`, <HiOutlineLightningBolt size="18px" />, takeSnapshot), []);
    const viewBucketSnapshotsAction = useMemo(() => new Action(`${messages.viewColdSnapshots}`, <MdRestore size="18px" />, viewBucketSnapshots), []);
    const viewBucketVersionsAction = useMemo(() => new Action(`${messages.viewBucketVersions}`, <MdRestore size="18px" />, viewBucketVersions), []);
    const renameAction = useMemo(() => new Action(`${messages.rename}`, <FiEdit size="18px" />, rename), []);
    const createFolderAction = useMemo(() => new Action(`${messages.createNewFolder}`, <PiFolderNotchPlusBold size="18px" />, createFolder), []);
    const restoreColdVersionAction = useMemo(() => new Action(`${messages.restoreCold}`, <MdOutlineRestoreFromTrash size="18px" />, retoreColdVersion), []);
    const deleteHotDatadAction = useMemo(() => new Action(`${messages.deleteHotData}`, <BsBoxSeam size="18px" />, deleteHotData), []);
    const deletedAction = useMemo(() => new Action(`${messages.delete}`, <FiTrash2 size="18px" />, deleteBucket), []);
    const purgeAction = useMemo(() => new Action(`${messages.purgeColdKeys}`, <FiTrash2 size="18px" />, purgeColdKeys), []);

    const hotInrecactiveActions = [
        createFolderAction, uploadAction, createSnapshotAction, viewBucketSnapshotsAction, renameAction, deletedAction
    ];
    const warmInrecactiveActions = [
        createFolderAction, uploadAction, createSnapshotAction, restoreColdVersionAction, viewBucketVersionsAction, deleteHotDatadAction, purgeAction
    ];
    const coldIntecactiveActions = [
        createFolderAction, viewBucketSnapshotsAction, renameAction, viewBucketVersionsAction, purgeAction
    ];
    const hotBackupActions = [
        createSnapshotAction, renameAction, viewBucketSnapshotsAction, deletedAction
    ];
    const warmBackupActions = [
        viewBucketSnapshotsAction, createSnapshotAction, restoreColdVersionAction, viewBucketVersionsAction, deleteHotDatadAction, purgeAction
    ];
    const coldBackupActions = [
        viewBucketSnapshotsAction, restoreColdVersionAction, renameAction, purgeAction
    ];

    const actions: Record<string, Action[]> = {
        interactive_hot: hotInrecactiveActions,
        interactive_warm: warmInrecactiveActions,
        interactive_cold: coldIntecactiveActions,
        backup_hot: hotBackupActions,
        backup_warm: warmBackupActions,
        backup_cold: coldBackupActions,
    }

    return (
        <div className={`w-52 text-xs font-medium bg-white rounded-xl shadow-md z-10 select-none text-gray-900`}>
            {
                actions[bucketType].map(action =>
                    <div
                        key={action.label}
                        className="w-full flex items-center gap-2 py-2 px-3 border-b-1 border-gray-200 transition-all hover:bg-slate-200"
                        onClick={action.value}
                    >
                        {action.icon} {action.label}
                    </div>
                )
            }
        </div>
    );
};
