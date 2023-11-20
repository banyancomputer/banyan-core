import React, { useEffect, useRef, useState } from 'react';

import { localeToAlpha2CountryCode, localeToLanguage } from '@app/utils/locales';
import { useIntl } from 'react-intl';
import { locales } from '@app/App';
import { setLocalStorageItem } from '@app/utils/localStorage';
import { popupClickHandler } from '@/app/utils';

import { ChevronUp } from '@static/images/common';

export const LanguageSelect = () => {
    const selectRef = useRef<HTMLDivElement | null>(null);
    const [isOptionstVisible, setIsOptionsVisible] = useState(false);
    const { locale } = useIntl();

    const toggleSelect = () => {
        setIsOptionsVisible(prev => !prev);
    };

    const changeLanguage = (language: string) => {
        setLocalStorageItem('lang', language);
        window.dispatchEvent(new Event('storage'));
        toggleSelect();
    };

    const stopPropagation = (event: React.MouseEvent<HTMLUListElement>) => {
        event.stopPropagation();
    };

    useEffect(() => {
        const listener = popupClickHandler(selectRef.current!, setIsOptionsVisible);
        document.addEventListener('click', listener);

        return () => document.removeEventListener('click', listener);
    }, [selectRef]);

    return (
        <div
            ref={selectRef}
            onClick={toggleSelect}
            className="relative p-2.5 flex w-80 justify-between items-center gap-2 text-sm font-medium border-1 border-border-darken rounded-lg shadow-sm cursor-pointer select-none"
        >
            <img
                width={22}
                height={16}
                alt="Flag"
                src={`http://purecatamphetamine.github.io/country-flag-icons/3x2/${localeToAlpha2CountryCode[locale || '']}.svg`}
            />
            <span className="flex-grow">
                {localeToLanguage[locale || 'en']}
            </span>
            <span className={`${isOptionstVisible ? 'rotate-0' : 'rotate-180'}`}>
                <ChevronUp />
            </span>
            {isOptionstVisible &&
                <ul
                    onClick={stopPropagation}
                    className="absolute left-0 top-12 w-full max-h-48 overflow-y-auto bg-mainBackground border-1 border-border-regular rounded-lg shadow-sm z-10"
                >
                    {locales?.map((language: string) =>
                        <div
                            className="flex items-center gap-2 p-2.5 transition-all bg-secondaryBackground hover:bg-bucket-bucketHoverBackground cursor-pointer"
                            key={language}
                            onClick={() => changeLanguage(language)}
                        >
                            <img
                                width={22}
                                height={16}
                                alt={language}
                                src={`http://purecatamphetamine.github.io/country-flag-icons/3x2/${localeToAlpha2CountryCode[language]}.svg`}
                            />
                            {localeToLanguage[language]}
                        </div>
                    )}
                </ul>
            }
        </div>
    );
};
