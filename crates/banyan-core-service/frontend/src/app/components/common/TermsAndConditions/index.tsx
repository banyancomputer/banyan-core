import { useState } from 'react';
import { useIntl } from 'react-intl';

import { AccountType } from './AccountType';

import { useModal } from '@app/contexts/modals';
import { TermsAndColditionsClient } from '@/api/termsAndConditions';
import { User } from '@/entities/user';

import folders from "@static/images/termsAndConditions/folders.png";
import { Done, Logo } from '@static/images/common';

const termsClient = new TermsAndColditionsClient();

export const TermaAndConditions: React.FC<{ userData: User, acceptTerms: React.Dispatch<React.SetStateAction<boolean>> }> = ({ userData, acceptTerms }) => {
    const [accountType, setAccountType] = useState('');
    const [areTermsAccepted, setAreTermsAccepted] = useState(false);
    const { closeModal } = useModal();
    const { messages } = useIntl();
    const isFormFilled = accountType && areTermsAccepted;

    const submit = async () => {
        const accepted_tos_at = Math.trunc(Date.now() / 1000);

        try {
            await termsClient.confirmTermsAndConditions(userData, accepted_tos_at);
            acceptTerms(true);
            closeModal();
        } catch (error: any) { }
    }

    return (
        <section className="w-screen h-screen flex flex-col">
            <div className="px-16 py-8 bg-termsAndConditions-header">
                <Logo />
            </div>
            <div className="flex-grow flex items-stretch">
                <div className="w-1/2 pt-24 flex flex-col items-center bg-white">
                    <div className="flex flex-col">
                        <h4 className="text-[32px] font-semibold">
                            {`${messages.whatAreYouUsingBanyanFor}`}
                        </h4>
                        <div className="mt-6 mb-8 flex items-center gap-3">
                            <AccountType
                                action={setAccountType}
                                isActive={accountType === 'work'}
                                text={'For work'}
                                value='work'
                            />
                            <AccountType
                                action={setAccountType}
                                isActive={accountType === 'personal'}
                                text={'Personal use'}
                                value='personal'
                            />
                        </div>
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
                                    {`${messages.termsOf}`}
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
                        <button
                            className="btn-primary max-w-[428px] h-12 py-2 px-4 mt-10 bg-navigation-primary text-white rounded-md font-bold text-sm"
                            disabled={!isFormFilled}
                            onClick={submit}
                        >
                            {`${messages.continue}`}
                        </button>
                    </div>
                </div>
                <div className="w-1/2 flex justify-center items-center bg-termsAndConditions-highlight">
                    <img
                        src={folders}
                        alt="Folders icon"
                        className="w-[372px]"
                    />
                </div>
            </div>
        </section>
    )
};
