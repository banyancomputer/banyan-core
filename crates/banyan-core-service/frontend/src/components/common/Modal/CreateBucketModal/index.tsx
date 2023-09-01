import React, { useMemo, useState } from 'react';
import { useIntl } from 'react-intl';

import { useModal } from '@/contexts/modals';
import { useTomb } from '@/contexts/tomb';
import { Select, Selectoption } from '../../Select';

export const CreateBucketModal = () => {
    const { closeModal } = useModal();
    const { messages } = useIntl();
    const [bucketName, setBucketName] = useState('');
    const { createBucket } = useTomb();
    const [bucketType, setBucketType] = useState('');
    const [storageClass, setStorageClass] = useState('');
    const isBucketDataFilled = useMemo(() =>
        !!bucketType && !!storageClass && !!bucketName,
        [bucketName, storageClass, bucketName])

    const bucketTypes = [
        new Selectoption('Hot', 'hot'),
        new Selectoption('Warm', 'warm'),
        new Selectoption('Cold', 'cold'),
    ];

    const storageClasses = [
        new Selectoption('Interactive', 'interactive'),
        new Selectoption('Backup', 'backup'),
    ];

    const selectBucketType = (option: string) => {
        setBucketType(option);
    };
    const selectStorageClass = (option: string) => {
        setStorageClass(option);
    };

    const create = async () => {
        try {
            createBucket(bucketName, storageClass, bucketType)
        } catch (error: any) {
            console.log('createBucketError', error);
        };
    };

    return (
        <div className="w-modal flex flex-col gap-5" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.createNewBucket}`}</h4>
            </div>
            <div>
                <label>
                    {`${messages.bucketName}`}
                    <input
                        className="mt-2 input w-full h-11 py-3 px-4 rounded-lg border-gray-200 focus:outline-none"
                        type="text"
                        placeholder={`${messages.enterNewBucketName}`}
                        value={bucketName}
                        onChange={event => setBucketName(event.target.value)}
                    />
                </label>
            </div>
            <div>
                <label className="inline-block mb-1 text-xs font-normal">{`${messages.bucketType}`}:</label>
                <Select
                    selectedOption={bucketType}
                    onChange={selectBucketType}
                    options={bucketTypes}
                    placeholder={`${messages.bucketType}`}
                />
            </div>
            <div>
                <label className="inline-block mb-1 text-xs font-normal">{`${messages.storageClass}`}:</label>
                <Select
                    selectedOption={storageClass}
                    onChange={selectStorageClass}
                    options={storageClasses}
                    placeholder={`${messages.storageClass}`}
                />
            </div>
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
