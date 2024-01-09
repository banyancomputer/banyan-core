import { useTomb } from '@app/contexts/tomb';

export const ErrorBanner = () => {
    const { error } = useTomb();

    return (
        <>
            {error ?
                <div className="flex items-center gap-3 py-4 px-2.5 bg-errorBanner  border-errorBannerBorder text-xs font-semibold text-error">
                    {error}
                </div>
                :
                null
            }
        </>
    );
};
