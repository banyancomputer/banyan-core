import { useIntl } from 'react-intl';

export const BetaBanner = () => {
    const { messages } = useIntl();

    return (
        <div className="flex items-center gap-2 p-4 bg-betaBanner border-2 border-betaBannerBorder text-xs leading-3 tracking-tighter font-semibold text-text-900">
            {`${messages.betaBannerWarning}`}
        </div>
    );
};
