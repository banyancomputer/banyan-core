import React, { useEffect, useState } from 'react';
import { HiOutlineLightningBolt } from 'react-icons/hi';
import { useIntl } from 'react-intl';
import { useForm } from 'react-hook-form';

import { Input } from '../../Input';

import { useKeystore } from '@/app/contexts/keystore';
import { validateKeyphrase } from '@/app/utils/validation';
import { useModal } from '@/app/contexts/modals';

export const CreateSecretKeyModal = () => {
    const { messages } = useIntl();
    const { initializeKeystore } = useKeystore();
    const { closeModal } = useModal();
    const {
        formState: { errors },
        handleSubmit,
        register,
        getValues,
        trigger,
        watch,
    } = useForm({
        mode: 'all',
        values: { keyphrase: '', keyphraseConfirmation: '' },
    });
    const { keyphrase, keyphraseConfirmation } = watch();
    const isDataCorrect = !Object.keys(errors).length && !!keyphrase && !!keyphraseConfirmation && keyphraseConfirmation === keyphrase;

    const confirm = async() => {
        try {
            await initializeKeystore(keyphrase);
            closeModal();
        } catch (error: any) {
            console.log(`Failed to initialize keystore: ${error.message}`);
        };
    };

    useEffect(() => {
        if (!keyphraseConfirmation) { return; }

        trigger('keyphraseConfirmation');
    }, [keyphrase, keyphraseConfirmation]);

    return (
        <form
            className="w-modal flex flex-col gap-8"
            onSubmit={handleSubmit(confirm)}
        >
            <span className="p-3 w-min rounded-full bg-gray-200">
                <HiOutlineLightningBolt size="24px" />
            </span>
            <div>
                <h4 className="text-m font-semibold">{`${messages.createSecretKey}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.andEnterIntoTextField}`}
                </p>
            </div>
            <Input
                type="password"
                label={`${messages.secretKey}`}
                placeholder={`${messages.enterPassphrase}`}
                error={errors.keyphrase?.message}
                register={register('keyphrase', {
                    required: `${messages.enterPassphrase}`,
                    validate: validateKeyphrase(`${messages.keyRequirements}`),
                })}
            />
            <Input
                type="password"
                label={`${messages.confirmSecretKey}`}
                placeholder={`${messages.enterPassphrase}`}
                error={errors.keyphraseConfirmation?.message}
                register={register('keyphraseConfirmation', {
                    required: `${messages.enterPassphrase}`,
                    validate: (keyphraseConfirmation) => {
                        const { keyphrase } = getValues();
                        if (keyphrase !== keyphraseConfirmation) {
                            return `${messages.passphraseNotMatch}`;
                        }

                        return validateKeyphrase(`${messages.keyRequirements}`)(keyphraseConfirmation);
                    },
                })}
            />
            <button
                type="submit"
                className="btn-primary flex-grow py-2.5 px-4"
                disabled={!isDataCorrect}
            >
                {`${messages.confirm}`}
            </button>
        </form >
    );
};
