import React, { useEffect, useRef, useState } from 'react'
import { useIntl } from 'react-intl';

import { CompanyNameModal } from './CompanyName'
;
import { TermsAndConditions } from '@app/types/terms';
import { useModal } from '@app/contexts/modals';

export const TermsAndConditionsModal: React.FC<{ terms: TermsAndConditions }> = ({ terms }) => {
    const { messages } = useIntl();
    const { openModal } = useModal();
    const [isTermsRead, setIsTermsRead] = useState(false);
    const termsRef = useRef<HTMLDivElement | null>(null);

    const acceptTerms = () => {
        openModal(<CompanyNameModal />, null, true);
    }

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
        <div className="p-1 w-termsAndConditions h-termsAndConditions flex flex-col gap-6">
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
                    className="w-full p-6 text-xs max-w-termsAndConditionsText whitespace-break-spaces"
                >
                    {terms.text}
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
