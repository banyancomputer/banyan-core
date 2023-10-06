import React, { useMemo, useState } from 'react';
import { useIntl } from 'react-intl';
import { useForm } from 'react-hook-form';

import { Select, Selectoption } from '../../Select';
import { Input } from '../../Input';

import { useModal } from '@/contexts/modals';
import { useTomb } from '@/contexts/tomb';
import { ToastNotifications } from '@/utils/toastNotifications';

export const CreateBucketModal = () => {
    const { closeModal } = useModal();
    const { messages } = useIntl();
    const { createBucket, buckets } = useTomb();
    const [bucketType, setBucketType] = useState('interactive');

    const {
        formState: { errors },
        handleSubmit,
        register,
        watch,
    } = useForm({
        mode: 'onChange',
        values: { bucketName: '' },
    });

    const { bucketName } = watch();
    const regexp = new RegExp(/^.{3,32}$/);
    const isBucketDataFilled = useMemo(() => regexp.test(bucketName) && !errors.bucketName, [bucketName, errors.bucketName]);
    const bucketsNames = buckets.map(bucket => bucket.name);
    const bucketTypes = [
        new Selectoption('Interactive', 'interactive'),
        new Selectoption('Backup', 'backup'),
    ];

    const selectBucketType = (option: string) => {
        setBucketType(option);
    };

    const validateBucketName = (name: string) => {
        if (!regexp.test(name)) return `${messages.bucket} ${messages.nameLengthError}`;
        if (bucketsNames.includes(name)) return `${messages.bucket} ${messages.nameDuplicationError}`
    };

    const create = async () => {
        try {
            await createBucket(bucketName, 'hot', bucketType);
            closeModal();
        } catch (error: any) {
            ToastNotifications.error(`${messages.creationError}`, `${messages.tryAgain}`, create);
        };
    };

    return (
        <form
            className="w-modal flex flex-col gap-5"
            onSubmit={handleSubmit(create)}
        >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.createNewBucket}`}</h4>
            </div>
            <div>
                <label>
                    {`${messages.bucketName}`}
                    <Input
                        placeholder={`${messages.enterNewBucketName}`}
                        error={errors.bucketName?.message}
                        register={register('bucketName', {
                            required: `${messages.enterNewBucketName}`,
                            validate: validateBucketName,
                        })}
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
            <div className="flex items-center gap-3 text-xs" >
                <button
                    className="btn-secondary flex-grow py-3 px-4"
                    onClick={closeModal}
                >
                    {`${messages.cancel}`}
                </button>
                <button
                    className="btn-primary flex-grow py-3 px-4"
                    disabled={!isBucketDataFilled}
                >
                    {`${messages.create}`}
                </button>
            </div>
        </form >
    );
};
