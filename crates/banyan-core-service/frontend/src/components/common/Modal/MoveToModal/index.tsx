import React, { useState } from 'react';
import { useIntl } from 'react-intl';
import { MdDone } from 'react-icons/md';

import { useModal } from '@/contexts/modals';
import { Bucket, BucketFile } from '@/lib/interfaces/bucket';
import { ToastNotifications } from '@/utils/toastNotifications';
import { useTomb } from '@/contexts/tomb';

import { Select } from '../../Select';
import { CreateBucketModal } from '../CreateBucketModal';
import { CreateFolderModal } from '../CreateFolderModal ';
import { UploadFileModal } from '../UploadFileModal';
import { AddNewOption } from '../../Select/AddNewOption';
import { useFolderLocation } from '@/hooks/useFolderLocation';
import { FolderSelect } from '../../FolderSelect';

export const MoveToModal: React.FC<{ file: BucketFile, bucket: Bucket }> = ({ file, bucket }) => {
    const { messages } = useIntl();
    const { moveTo } = useTomb();
    const { closeModal, openModal } = useModal();
    const [selectedBucket, setSelectedBucket] = useState('');
    const [selectedFolder, setSelectedFolder] = useState<string[]>([]);
    const folderLocation = useFolderLocation();

    const move = async () => {
        try {
            await moveTo(bucket, [...folderLocation, file.name], [...selectedFolder, selectedBucket, file.name])
            ToastNotifications.notify(`${messages.fileWasMoved}`, <MdDone size="20px" />);
        } catch (error: any) {
            ToastNotifications.error(`${messages.moveToError}`, `${messages.tryAgain}`, move);

        };
    };

    // const selectBucket = (option: string) => {
    //     setSelectedBucket(option);
    // };

    const selectFolder = (option: string[]) => {
        setSelectedFolder(option);
    };

    // const addNewBucket = () => {
    //     openModal(<CreateBucketModal />, () => openModal(<MoveToModal bucket={bucket} file={file} />));
    // };

    return (
        <div className="w-modal flex flex-col gap-6" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.moveTo}`}</h4>
                <p className="mt-2 text-gray-600">
                    {`${messages.selectWhereToMove}`}
                </p>
            </div>
            {/* <div>
                <label className="inline-block mb-1 text-xs font-normal">{`${messages.selectInTheList}`}:</label>
                <Select
                    selectedOption={selectedBucket}
                    onChange={selectBucket}
                    options={buckets.map(bucket => ({ value: bucket.id, label: bucket.name }))}
                    placeholder={`${messages.selectBucket}`}
                    initialOption={<AddNewOption label={`${messages.createNewBucket}`} action={addNewBucket} />}
                />
            </div> */}
            <div>
                <label className="inline-block mb-1 text-xs font-normal">{`${messages.folder}`}:</label>
                <FolderSelect
                    selectedBucket={selectedBucket}
                    onChange={selectFolder}
                    onFolderCreation={() => openModal(<MoveToModal bucket={bucket} file={file} />)}
                />
            </div>
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <button
                    className="btn-secondary flex-grow py-3 px-4"
                    onClick={closeModal}
                >
                    {`${messages.cancel}`}
                </button>
                <button
                    className="btn-primary flex-grow py-3 px-4"
                    onClick={move}
                >
                    {`${messages.moveTo}`}
                </button>
            </div>
        </div>
    );
};
