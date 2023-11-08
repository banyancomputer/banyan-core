import { useTomb } from '@app/contexts/tomb';
import React from 'react';
import { AiOutlineWarning } from 'react-icons/ai';

export const ErrorBanner = () => {
    const { error, setError } = useTomb();

    return (
        <>
            {error ?
                <div className="flex justify-center items-center gap-3 py-4 px-2.5 bg-errorBanner border-2 border-errorBannerBorder text-sm font-medium">
                    <AiOutlineWarning size="20px" />
                    {error}
                </div>
                :
                null
            }
        </>
    )
};