import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import { unwrapResult } from '@reduxjs/toolkit';
import { useEffect, useState } from 'react';

import { Input } from '@components/common/Input';
import { PrimaryButton } from '@components/common/PrimaryButton';

import { useAppDispatch, useAppSelector } from '@app/store';
import { initializeKeystore } from '@store/keystore/actions';
import { TermsAndColditionsClient } from '@/api/termsAndConditions';
import { UserClient } from '@/api/user';

import { Done } from '@static/images/common';

const termsClient = new TermsAndColditionsClient();
const userClient = new UserClient();

const EnterEncryptionKey = () => {
    const messages = useAppSelector(state => state.locales.messages.pages.enterEncryptionKey);
    const { agreeToTerms, termsOfService, and, privacyPolicy } = useAppSelector(state => state.locales.messages.pages.createEncryptionKey);
    const [isTermsCheckboxVisible, setIsTermsCheckboxVisible] = useState(false);
    const [areTermsAccepted, setAreTermsAccepted] = useState(false);
    const navigate = useNavigate();
    const dispatch = useAppDispatch();
    const {
        formState: { errors },
        handleSubmit,
        register,
        setError,
        watch,
    } = useForm({
        mode: 'all',
        values: { keyphrase: '' },
    });
    const { keyphrase } = watch();
    const isDataCorrect = !Object.keys(errors).length && keyphrase.length >= 8 && (isTermsCheckboxVisible ? areTermsAccepted : true);

    const confirm = async () => {
        try {
            unwrapResult(await dispatch(initializeKeystore(keyphrase)));
            navigate('/');
        } catch (error: any) {
            setError('keyphrase', { message: `${messages.secretKeyError}` });
        };
    };

    useEffect(() => {
        (async () => {
            const termsAndConditions = await termsClient.getTermsAndCondition();
            const user = await userClient.getCurrentUser();
            if (!user.acceptedTosAt || (user.acceptedTosAt <= +termsAndConditions.tos_date)) {
                setIsTermsCheckboxVisible(true)
            }
        })();
    }, [])

    return (
        <form
            className="flex flex-col gap-4 w-[428px] max-w-[428px]"
            onSubmit={handleSubmit(confirm)}
        >
            <h2 className="text-[32px] font-semibold">{messages.title}</h2>
            <Input
                type="password"
                label={`${messages.encryptionKey}`}
                placeholder={`${messages.encryptionKeyPlaceholder}`}
                error={errors.keyphrase?.message}
                labelClassName="font-semibold"
                register={register('keyphrase', {
                    required: `${messages.encryptionKeyPlaceholder}`,
                })}
            />
            {isTermsCheckboxVisible ?
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
                        {`${agreeToTerms}`}
                        <a
                            href="https://www.banyan.computer/terms-of-service"
                            target="_blank"
                            className="underline"
                        >
                            {`${termsOfService}`}
                        </a>
                        {`${and}`}
                        <a
                            href="https://www.banyan.computer/privacy-policy"
                            target="_blank"
                            className="underline"
                        >
                            {`${privacyPolicy}`}
                        </a>
                    </div>
                </div>
                :
                null
            }
            {/* <div className="flex items-center gap-1.5 text-[13px] font-medium whitespace-nowrap text-button-primary ">
                {`${messages.forgotEncryptionKey}?`}
                <button className="underline font-semibold hover:text-button-primaryHover">
                    {messages.resetKey}
                </button>
            </div> */}
            <PrimaryButton
                action={confirm}
                text={messages.continue}
                disabled={!isDataCorrect}
            />
        </form>
    )
};

export default EnterEncryptionKey;
