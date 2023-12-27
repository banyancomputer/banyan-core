import React, { useMemo, useState } from 'react';
import { useIntl } from 'react-intl';
import { useNavigate } from 'react-router-dom';

import { SubmitButton } from '@components/common/SubmitButton';

import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';

export const CreateBucketModal = () => {
    const { closeModal } = useModal();
    const navigate = useNavigate();
    const { messages } = useIntl();
    const [bucketName, setBucketName] = useState('');
    const { createBucketAndMount } = useTomb();
    const [bucketType, setBucketType] = useState('interactive');
    const [storageClass, setStorageClass] = useState('hot');
    const isBucketDataFilled = useMemo(() =>
        !!bucketType && !!(bucketName.length >= 3),
    [bucketName, bucketName]);

    const changeBucketName = (event: React.ChangeEvent<HTMLInputElement>) => {
        const regexp = new RegExp(/^.{0,32}$/);
        if (!regexp.test(event.target.value)) { return; }

        setBucketName(event.target.value);
    };

    const create = async() => {
        try {
            const bucketId = await createBucketAndMount(bucketName, storageClass, bucketType);
            closeModal();
            navigate(`/drive/${bucketId}`);
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
                <SubmitButton
                    text={`${messages.create}`}
                    action={create}
                    disabled={!isBucketDataFilled}
                />
            </div>
        </div >
    );
};
