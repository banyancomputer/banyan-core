import React, { useEffect, useRef, useState } from 'react';

import { localeToAlpha2CountryCode, localeToLanguage } from '@utils/locales';
import { LANGUAGES, LANGUAGES_KEYS, changeLanguage } from '@/app/store/locales/slice';
import { setLocalStorageItem } from '@utils/localStorage';
import { popupClickHandler } from '@/app/utils';

import { ChevronUp } from '@static/images/common';
import { useAppDispatch, useAppSelector } from '@/app/store';

export const LanguageSelect = () => {
    const { key } = useAppSelector(state => state.locales);
    const selectRef = useRef<HTMLDivElement | null>(null);
    const dispatch = useAppDispatch();
    const [isOptionstVisible, setIsOptionsVisible] = useState(false);

    const toggleSelect = () => {
        setIsOptionsVisible(prev => !prev);
    };

    const handleLanguageChange = (language: string) => {
        dispatch(changeLanguage(language as LANGUAGES_KEYS));
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
                src={`http://purecatamphetamine.github.io/country-flag-icons/3x2/${localeToAlpha2CountryCode[key]}.svg`}
            />
            <span className="flex-grow">
                {localeToLanguage[key]}
            </span>
            <span className={`${isOptionstVisible ? 'rotate-0' : 'rotate-180'}`}>
                <ChevronUp />
            </span>
            {isOptionstVisible &&
                <ul
                    onClick={stopPropagation}
                    className="absolute left-0 top-12 w-full max-h-48 overflow-y-auto bg-mainBackground border-1 border-border-regular rounded-lg shadow-sm z-10"
                >
                    {Object.keys(LANGUAGES).map((language) =>
                        <div
                            className="flex items-center gap-2 p-2.5 transition-all bg-secondaryBackground hover:bg-bucket-bucketHoverBackground cursor-pointer"
                            key={language}
                            onClick={() => handleLanguageChange(language)}
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
