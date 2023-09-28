import React, { useState } from 'react';
import { HiOutlineLightningBolt } from 'react-icons/hi';
import { useIntl } from 'react-intl';
import { useForm } from 'react-hook-form';

import { PasswordInput } from '../../PasswordInput';

import { useKeystore } from '@/contexts/keystore';
import { useModal } from '@/contexts/modals';
import { validateKeyphrase } from '@/utils/validation';

export const EnterSecretKeyModal = () => {
    const { messages } = useIntl();
    const { initializeKeystore } = useKeystore();
    const { closeModal } = useModal();
    const {
        formState: { errors },
        handleSubmit,
        register,
        setError,
        watch,
    } = useForm({
        mode: 'onTouched',
        values: { keyphrase: '' },
    });
    const { keyphrase } = watch();

    const confirm = async () => {
        try {
            await initializeKeystore(keyphrase);
            closeModal();
        } catch (error: any) {
            /** TODO: rework when error message from tomb will be more specific. */
            setError('keyphrase', { message: `${messages.wrongSecretKey}` })
        };
    };

    return (
        <form
            className="w-modal flex flex-col gap-8"
            onSubmit={handleSubmit(confirm)}
        >
            <span className="p-3 w-min rounded-full bg-gray-200">
                <HiOutlineLightningBolt size="24px" />
            </span>
            <div>
                <h4 className="text-m font-semibold">{`${messages.inputSecretKey}`}</h4>
                <p className="mt-2 text-gray-600">
                    {`${messages.enterSecretKey}`}
                </p>
            </div>
            <div>
                <label className="inline-block mb-1 text-xs font-normal">{`${messages.secretKey}`}</label>
                <PasswordInput
                    placeholder={`${messages.enterPassphrase}`}
                    error={errors.keyphrase?.message}
                    register={register('keyphrase', {
                        required: `${messages.enterPassphrase}`,
                        validate: validateKeyphrase(`${messages.keyRequirements}`),
                    })}
                />
            </div>
            <button
                type="submit"
                className="btn-primary flex-grow py-2.5 px-4"
                disabled={!keyphrase}
            >
                {`${messages.confirm}`}
            </button>
        </form >
    );
};
