import { useAppSelector } from "@/app/store";

export const BetaBanner = () => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.betaBanner);

    return (
        <div className="flex items-center gap-2 p-4 bg-betaBanner border-2 border-betaBannerBorder text-xs leading-3 tracking-tighter font-semibold text-text-900">
            {`${messages.title}`}
        </div>
    );
};
