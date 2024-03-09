import React, { useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { SecondaryButton } from '@components/common/SecondaryButton';

import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { ToastNotifications } from '@/app/utils/toastNotifications';
import { useAppSelector } from '@/app/store';

export const CreateBucketModal = () => {
    const { closeModal } = useModal();
    const navigate = useNavigate();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.createBucket);
    const { driveAlreadyExists } = useAppSelector(state => state.locales.messages.contexts.tomb);
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

    const create = async () => {
        try {
            const bucketId = await createBucketAndMount(bucketName, storageClass, bucketType);
            closeModal();
            navigate(`/drive/${bucketId}`);
        } catch (error: any) {
            if(error.message !== driveAlreadyExists) {
                ToastNotifications.error(`${messages.creationError}`, `${messages.tryAgain}`, create);
            };
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
                        className="mt-2 input w-full h-11 py-3 px-4 rounded-md border-1 border-border-darken focus:outline-none"
                        type="text"
                        placeholder={`${messages.enterNewName}`}
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
            <div className="flex items-center justify-end gap-3 text-xs" >
                <SecondaryButton
                    action={closeModal}
                    text={`${messages.cancel}`}
                />
                <PrimaryButton
                    text={`${messages.create}`}
                    action={create}
                    disabled={!isBucketDataFilled}
                />
            </div>
        </div >
    );
};
