import { useIntl } from 'react-intl';

import { ErrorBannerIcon } from '@static/images/common';

export const BetaBanner = () => {
    const { messages } = useIntl();

    return (
        <div className="mb-2 flex justify-center items-center gap-3 py-4 px-2.5 bg-errorBanner border-2 border-errorBannerBorder text-xxs font-medium text-text-900">
            <ErrorBannerIcon />
            {`${messages.betaBannerWarning}`}
        </div>
    )
};