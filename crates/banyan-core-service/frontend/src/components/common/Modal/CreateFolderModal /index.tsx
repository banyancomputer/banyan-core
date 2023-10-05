import React, { useMemo, useState } from 'react';
import { useIntl } from 'react-intl';
import { useForm } from 'react-hook-form';

import { useModal } from '@/contexts/modals';
import { useTomb } from '@/contexts/tomb';
import { Bucket } from '@/lib/interfaces/bucket';
import { ToastNotifications } from '@/utils/toastNotifications';
import { Input } from '../../Input';

export const CreateFolderModal: React.FC<{ bucket: Bucket, onSuccess?: () => void, path: string[] }> = ({ bucket, onSuccess = () => { }, path }) => {
    const {
        formState: { errors },
        handleSubmit,
        register,
        watch,
    } = useForm({
        mode: 'onChange',
        values: { folderName: '' },
    });
    const { closeModal, openModal } = useModal();
    const { messages } = useIntl();

    const { createDirectory } = useTomb();
    const { folderName } = watch();
    const regexp = new RegExp(/^.{0,32}$/);
    const isFolderNameValid = useMemo(() => regexp.test(folderName) && !errors.folderName, [folderName, errors.folderName]);
    const filesNames = bucket.files.map(file => file.name);

    const validateFolderName = (name: string) => {
        if (!regexp.test(name)) return `${messages.folder} ${messages.nameLengthError}`;
        if (filesNames.includes(name)) return `${messages.folder} ${messages.nameDuplicationError}`
    };

    const changeName = (event: React.ChangeEvent<HTMLInputElement>) => {
        if (event.target.value.length >= 32) return;

        setfolderName(event.target.value);
    };

    const create = async () => {
        try {
            await createDirectory(bucket, path, folderName);
            onSuccess();
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
                <h4 className="text-m font-semibold ">{`${messages.createNewFolder}`}</h4>
            </div>
            <div>
                <label>
                    {`${messages.folderName}`}
                    <Input
                        placeholder={`${messages.enterNewBucketName}`}
                        error={errors.folderName?.message}
                        register={register('folderName', {
                            required: `${messages.enterNewBucketName}`,
                            validate: validateFolderName,
                        })}
                    />
                </label>
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
                    disabled={!isFolderNameValid}

                >
                    {`${messages.create}`}
                </button>
            </div>
        </form >
    );
};
