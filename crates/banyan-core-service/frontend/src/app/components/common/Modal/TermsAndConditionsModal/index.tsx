import React, { useEffect, useRef, useState } from 'react'
import { useIntl } from 'react-intl';

import { CompanyNameModal } from './CompanyName';

import { useModal } from '@app/contexts/modals';
import { TermsAndColditionsClient } from '@/api/termsAndConditions';
import { RawUser } from '@app/types';

const termsClient = new TermsAndColditionsClient();

export const TermsAndConditionsModal: React.FC<{ terms: string, userData: RawUser }> = ({ terms, userData }) => {
    const { messages } = useIntl();
    const { openModal } = useModal();
    const [isTermsRead, setIsTermsRead] = useState(true);
    const termsRef = useRef<HTMLDivElement | null>(null);

    const acceptTerms = async () => {
        const accepted_tos_at = Math.trunc(Date.now() / 1000);

        try {
            await termsClient.confirmTermsAndConditions(userData, accepted_tos_at);
            openModal(<CompanyNameModal />, null, true);
        } catch (error: any) { }
    };

    useEffect(() => {
        if (!termsRef.current) return;

        const listener = () => {
            const preview = document.getElementById('file-preview');

            if (termsRef.current!.scrollTop >= preview!.clientHeight - termsRef.current!.clientHeight) {
                setIsTermsRead(true);
            }
        };

        termsRef.current?.addEventListener('scroll', listener);

        return () => termsRef.current?.removeEventListener('scroll', listener);
    }, [termsRef.current, isTermsRead]);

    return (
        <div className="p-1 w-[calc(100vw-100px)] max-w-termsAndConditions max-h-full flex flex-col gap-6">
            <h3 className="flex items-center justify-between text-m font-semibold">
                {`${messages.termsOfService}`}
                <span>1/2</span>
            </h3>
            <div
                ref={termsRef}
                className="border-1 rounded-lg border-border-regular overflow-y-scroll"
            >
                <div
                    id="file-preview"
                    className="w-full p-6 text-xs whitespace-break-spaces"
                >
                    {terms}
                </div>
            </div>
            <div className="flex items-center gap-3 text-xs">
                <button
                    className="btn-secondary w-1/2 py-3 px-4"
                >
                    {`${messages.decline}`}
                </button>
                <button
                    className="btn-primary w-1/2 py-3 px-4"
                    disabled={!isTermsRead}
                    onClick={acceptTerms}
                >
                    {`${messages.acceptTermsAndService}`}
                </button>
            </div>
        </div>
    )
}