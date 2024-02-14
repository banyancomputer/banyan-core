import { useError } from '@/app/contexts/error';
import { Close } from '@/app/static/images/common';

export const ErrorBanner = () => {
    const { errors, closeError } = useError();

    return (
        <>
            {errors.length ?
                <>
                    {errors.map((error, index) =>
                        <div
                            key={index}
                            className="relative flex items-center gap-3 py-4 px-2.5 bg-errorBanner  border-errorBannerBorder text-xs font-semibold text-error"
                        >
                            {error.message}
                            {error.action &&
                                <button
                                    className="text-text-900 underline cursor-pointer"
                                    onClick={error.action.callback}
                                >
                                    {error.action.label}
                                </button>
                            }
                            {error.canBeClosed &&
                                <button
                                    className="absolute top-1/2 -translate-y-1/2 right-4 text-text-900 cursor-pointer"
                                    onClick={() => closeError(error)}
                                >
                                    <Close width="24px" height="24px" />
                                </button>
                            }
                        </div>
                    )}
                </>
                :
                null
            }
        </>
    );
};
