import React, { useState } from 'react';
import { HiOutlineLightningBolt } from 'react-icons/hi';
import { useIntl } from 'react-intl';
import { useForm } from 'react-hook-form';
import Router from 'next/router';

import { PasswordInput } from '../../PasswordInput';

import { useKeystore } from '@/contexts/keystore';
import { validateKeyphrase } from '@/utils/validation';
import { useModal } from '@/contexts/modals';

export const CreateSecretKeyModal = () => {
    const { messages } = useIntl();
    const { initializeKeystore } = useKeystore();
    const { closeModal } = useModal()
    const {
        formState: { errors },
        handleSubmit,
        register,
        getValues,
        watch
    } = useForm({
        mode: 'onTouched',
        values: { keyphrase: '', keyphraseConfirmation: '' },
    });
    const { keyphrase, keyphraseConfirmation } = watch();
    const isDataCorrect = !Object.keys(errors).length && !!keyphrase && !!keyphraseConfirmation && keyphraseConfirmation === keyphrase;

    const confirm = async () => {
        try {
            await initializeKeystore(keyphrase);
            Router.reload();
            closeModal();
        } catch (error: any) {
            console.log(`Failed to initialize keystore: ${error.message}`);
        };
    };

    return (
        <form
            className='w-modal flex flex-col gap-8'
            onSubmit={handleSubmit(confirm)}
        >
            <span className="p-3 w-min rounded-full bg-gray-200">
                <HiOutlineLightningBolt size="24px" />
            </span>
            <div>
                <h4 className="text-m font-semibold">{`${messages.createSecretKey}`}</h4>
                <p className="mt-2 text-gray-600">
                    {`${messages.andEnterIntoTextField}`}
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
            <div>
                <label className="inline-block mb-1 text-xs font-normal">{`${messages.confirmSecretKey}`}</label>
                <PasswordInput
                    placeholder={`${messages.enterPassphrase}`}
                    error={errors.keyphraseConfirmation?.message}
                    register={register('keyphraseConfirmation', {
                        required: `${messages.enterPassphrase}`,
                        validate: (keyphraseConfirmation) => {
                            const { keyphrase } = getValues();
                            if (keyphrase !== keyphraseConfirmation) {
                                return `${messages.passphraseNotMatch}`
                            }

                            return validateKeyphrase(`${messages.keyRequirements}`)(keyphraseConfirmation);
                        }
                    })}
                />
            </div>
            <button
                type='submit'
                className="btn-primary flex-grow py-select px-4"
                onClick={confirm}
                disabled={!isDataCorrect}
            >
                {`${messages.confirm}`}
            </button>
        </form >
    )
}
