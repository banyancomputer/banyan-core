import React from 'react';
import { useIntl } from 'react-intl';
import { FiEdit, FiTrash2, FiUpload } from 'react-icons/fi';
import { HiOutlineLightningBolt } from 'react-icons/hi';
import { MdRestore } from 'react-icons/md';
import { BsBoxSeam } from 'react-icons/bs';

import { FileAction } from '../../Buckets/FileActions';
import { Bucket } from '@/lib/interfaces/bucket';
import { useModal } from '@/contexts/modals';
import { BucketSnapshotsModal } from '@/components/common/Modal/BucketSnapshotsModal';
import { RenameBucketModal } from '@/components/common/Modal/RenameBucketModal';
import { DeleteBucketModal } from '@/components/common/Modal/DeleteBucketModal';
import { TakeSnapshotModal } from '@/components/common/Modal/TakeSnapshotModal';

export const BucketActions: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const { messages } = useIntl();
    const { openModal } = useModal();

    const download = async() => {
        try {

        } catch (error: any) { }
    };

    const takeSnapshot = async() => {
        try {
            openModal(<TakeSnapshotModal bucket={bucket} />);
        } catch (error: any) { }
    };

    const viewBucketVersions = async() => {
        try {
            openModal(<BucketSnapshotsModal bucketId={bucket.id} />);
        } catch (error: any) { }
    };

    const rename = async() => {
        openModal(<RenameBucketModal bucket={bucket} />);
    };

    const deleteHotData = async() => {
        try {

        } catch (error: any) { }
    };

    const deleteBucket = async() => {
        try {
            openModal(<DeleteBucketModal bucket={bucket} />);
        } catch (error: any) { }
    };

    const acrions = [
        new FileAction(`${messages.upload}`, <FiUpload size="18px" />, download),
        new FileAction(`${messages.takeColdSnapshot}`, <HiOutlineLightningBolt size="18px" />, takeSnapshot),
        new FileAction(`${messages.viewBucketVersions}`, <MdRestore size="18px" />, viewBucketVersions),
        new FileAction(`${messages.rename}`, <FiEdit size="18px" />, rename),
        new FileAction(`${messages.deleteHotData}`, <BsBoxSeam size="18px" />, deleteHotData),
        new FileAction(`${messages.delete}`, <FiTrash2 size="18px" />, deleteBucket),
    ];

    return (
        <div className="relative w-52 text-xs font-medium bg-white rounded-xl shadow-md z-10 text-gray-900">{
            acrions.map(action =>
                <div
                    key={action.label}
                    className="w-full flex items-center gap-2 py-2 px-3 border-b-1 border-gray-200 transition-all hover:bg-slate-200"
                    onClick={action.value}
                >
                    {action.icon} {action.label}
                </div>
            )
        }</div>
    );
};
