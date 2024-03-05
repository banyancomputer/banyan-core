import React, { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';

import { PrimaryButton } from '@components/common/PrimaryButton';
import { Input } from '@components/common/Input';

import { useKeystore } from '@/app/contexts/keystore';
import { validateKeyphrase } from '@/app/utils/validation';
import { useModal } from '@/app/contexts/modals';
import { useAppSelector } from '@/app/store';

import { Bolt } from '@static/images/common';

export const CreateSecretKeyModal = () => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.modal.createSecretKey);
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

    const confirm = async () => {
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
            <div>
                <h4 className="text-m font-semibold">{`${messages.title}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.subtitle}`}
                </p>
            </div>
            <Input
                type="password"
                label={`${messages.secretKey}`}
                placeholder={`${messages.enterSecretKey}`}
                error={errors.keyphrase?.message}
                register={register('keyphrase', {
                    required: `${messages.enterSecretKey}`,
                    validate: validateKeyphrase(`${messages.keyRequirements}`),
                })}
            />
            <Input
                type="password"
                label={`${messages.confirmSecretKey}`}
                placeholder={`${messages.confirmSecretKey}`}
                error={errors.keyphraseConfirmation?.message}
                register={register('keyphraseConfirmation', {
                    required: `${messages.confirmSecretKey}`,
                    validate: (keyphraseConfirmation) => {
                        const { keyphrase } = getValues();
                        if (keyphrase !== keyphraseConfirmation) {
                            return `${messages.passphraseNotMatch}`;
                        }

                        return validateKeyphrase(`${messages.keyRequirements}`)(keyphraseConfirmation);
                    },
                })}
            />
            <PrimaryButton
                text={`${messages.confirm}`}
                disabled={!isDataCorrect}
                className="py-2.5"
            />
        </form >
    );
};
