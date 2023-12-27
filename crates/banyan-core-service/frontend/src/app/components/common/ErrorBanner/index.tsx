import { useTomb } from '@app/contexts/tomb';

import { ErrorBannerIcon } from '@static/images/common';

export const ErrorBanner = () => {
    const { error } = useTomb();

    return (
        <>
            {error ?
                <div className="flex justify-center items-center gap-3 py-4 px-2.5 bg-errorBanner border-2 border-errorBannerBorder text-sm font-medium text-text-900">
                    <ErrorBannerIcon />
                    {error}
                </div>
                :
                null
            }
        </>
    );
};
