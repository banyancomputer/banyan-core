import React, { useMemo, useState } from 'react';
import { useIntl } from 'react-intl';

import { Select, Selectoption } from '../../Select';
import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';

export const CreateBucketModal = () => {
    const { closeModal } = useModal();
    const { messages } = useIntl();
    const [bucketName, setBucketName] = useState('');
    const { createBucket } = useTomb();
    const [bucketType, setBucketType] = useState('interactive');
    const [storageClass, setStorageClass] = useState('hot');
    const isBucketDataFilled = useMemo(() =>
        !!bucketType && !!(bucketName.length >= 3),
    [bucketName, bucketName]);

    const bucketTypes = [
        new Selectoption('Interactive', 'interactive'),
        new Selectoption('Backup', 'backup'),
    ];

    const selectBucketType = (option: string) => {
        setBucketType(option);
    };

    const changeBucketName = (event: React.ChangeEvent<HTMLInputElement>) => {
        const regexp = new RegExp(/^.{0,32}$/);
        if (!regexp.test(event.target.value)) { return; }

        setBucketName(event.target.value);
    };

    const create = async () => {
        try {
            await createBucket(bucketName, storageClass, bucketType);
            closeModal();
        } catch (error: any) {
            ToastNotifications.error(`${messages.creationError}`, `${messages.tryAgain}`, create);
        };
    };

    return (
        <div className="w-modal flex flex-col gap-5" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.createNewDrive}`}</h4>
            </div>
            <div>
                <label>
                    {`${messages.driveName}`}
                    <input
                        className="mt-2 input w-full h-11 py-3 px-4 rounded-lg border-1 border-border-darken focus:outline-none"
                        type="text"
                        placeholder={`${messages.enterNewDriveName}`}
                        value={bucketName}
                        onChange={changeBucketName}
                    />
                </label>
            </div>
            {/* <div>
                <label className="inline-block mb-1 text-xs font-normal">{`${messages.driveType}`}:</label>
                <Select
                    selectedOption={bucketType}
                    onChange={selectBucketType}
                    options={bucketTypes}
                    placeholder={`${messages.driveType}`}
                />
            </div> */}
            <div className="flex items-center gap-3 text-xs" >
                <button
                    className="btn-secondary flex-grow py-3 px-4"
                    onClick={closeModal}
                >
                    {`${messages.cancel}`}
                </button>
                <button
                    className="btn-primary flex-grow py-3 px-4"
                    onClick={create}
                    disabled={!isBucketDataFilled}
                >
                    {`${messages.create}`}
                </button>
            </div>
        </div >
    );
};