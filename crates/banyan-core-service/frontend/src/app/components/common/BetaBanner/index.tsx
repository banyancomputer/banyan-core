import { useIntl } from 'react-intl';

import { ErrorBannerIcon } from '@static/images/common';

export const BetaBanner = () => {
    const { messages } = useIntl();

    return (
        <div className="mb-2 flex justify-center items-center gap-2 py-4 px-2 bg-errorBanner border-2 border-errorBannerBorder text-[10px] leading-3 tracking-tighter font-medium text-text-900">
            <ErrorBannerIcon width="14px" height="14px" />
            {`${messages.betaBannerWarning}`}
        </div>
    )
};