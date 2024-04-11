import React, { useEffect, useState } from 'react'
import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import { unwrapResult } from '@reduxjs/toolkit';

import { Input } from '@components/common/Input';
import { PrimaryButton } from '@components/common/PrimaryButton';

import { useAppDispatch, useAppSelector } from '@app/store';
import { UserClient } from '@/api/user';
import { validateKeyphrase } from '@utils/validation';
import { TermsAndColditionsClient } from '@/api/termsAndConditions';
import { initializeKeystore } from '../store/keystore/actions';

import { Done } from '@static/images/common';

const termsClient = new TermsAndColditionsClient();
const userClient = new UserClient();

const CreateEncryptionKey = () => {
    const messages = useAppSelector(state => state.locales.messages.pages.createEncryptionKey);
    const [areTermsAccepted, setAreTermsAccepted] = useState(false);
    const dispatch = useAppDispatch();
    const navigate = useNavigate();
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
    const isDataCorrect = !Object.keys(errors).length && !!keyphrase && !!keyphraseConfirmation && keyphraseConfirmation === keyphrase && areTermsAccepted;


    const confirm = async () => {
        const accepted_tos_at = Math.trunc(Date.now() / 1000);
        const userData = await userClient.getCurrentUser();
        try {
            /** TODO: disscuss account type filed which was removed in current design. */
            await termsClient.confirmTermsAndConditions(userData, accepted_tos_at, 'personal');
            unwrapResult(await dispatch(initializeKeystore(keyphrase)));
            navigate('/');
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
            className="flex flex-col gap-4 w-[428px] max-w-[428px]"
            onSubmit={handleSubmit(confirm)}
        >
            <div className="flex flex-col gap-3">
                <h2 className="text-[32px] font-semibold">{messages.title}</h2>
                <p className="text-xs">{messages.subtitle}</p>
            </div>
            <Input
                type="password"
                label={`${messages.newEncryptionKey}`}
                placeholder={`${messages.newEncryptionKeyPlaceholder}`}
                error={errors.keyphrase?.message}
                labelClassName="font-semibold"
                register={register('keyphrase', {
                    required: `${messages.newEncryptionKeyPlaceholder}`,
                    validate: validateKeyphrase(`${messages.keyRequirements}`),
                })}
            />
            <Input
                type="password"
                label={`${messages.reenterEncryptionKey}`}
                placeholder={`${messages.reenterEncryptionKeyPlaceholder}`}
                error={errors.keyphraseConfirmation?.message}
                labelClassName="font-semibold"
                register={register('keyphraseConfirmation', {
                    required: `${messages.reenterEncryptionKeyPlaceholder}`,
                    validate: (keyphraseConfirmation) => {
                        const { keyphrase } = getValues();
                        if (keyphrase !== keyphraseConfirmation) {
                            return `${messages.passphraseNotMatch}`;
                        }

                        return validateKeyphrase(`${messages.keyRequirements}`)(keyphraseConfirmation);
                    },
                })}
            />
            <div
                className="flex items-center gap-2"
            >
                <div
                    className="w-5 h-5 flex justify-center items-center border-1 border-border-darken rounded-sm text-termsAndConditions-activeAccountType cursor-pointer"
                    onClick={() => setAreTermsAccepted(prev => !prev)}
                >
                    {areTermsAccepted && <Done />}
                </div>
                <div className="flex items-center gap-1.5 text-[13px] font-medium whitespace-nowrap">
                    {`${messages.agreeToTerms}`}
                    <a
                        href="https://www.banyan.computer/terms-of-service"
                        target="_blank"
                        className="underline"
                    >
                        {`${messages.termsOfService}`}
                    </a>
                    {`${messages.and}`}
                    <a
                        href="https://www.banyan.computer/privacy-policy"
                        target="_blank"
                        className="underline"
                    >
                        {`${messages.privacyPolicy}`}
                    </a>
                </div>
            </div>
            <PrimaryButton
                action={confirm}
                text={messages.continue}
                disabled={!isDataCorrect}
            />
        </form>
    )
};

export default CreateEncryptionKey;
