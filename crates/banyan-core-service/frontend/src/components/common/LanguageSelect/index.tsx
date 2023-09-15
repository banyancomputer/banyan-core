import React, { ReactElement, useEffect, useRef, useState } from 'react';
import { FiChevronDown } from 'react-icons/fi';
import { useRouter } from 'next/router';
import Link from 'next/link';
import Image from 'next/image';

import { localeToAlpha2CountryCode, localeToLanguage } from '@utils/locales';
import { Selectoption } from '../Select';
import { popupClickHandler } from '@/utils';

export const LanguageSelect = () => {
    const selectRef = useRef<HTMLDivElement | null>(null);
    const [isOptionstVisible, setIsOptionsVisible] = useState(false);
    const { locales, locale } = useRouter();

    const toggleSelect = () => {
        setIsOptionsVisible(prev => !prev);
    };

    const handleSelect = (option: Selectoption) => {
        setIsOptionsVisible(false);
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
            className="relative p-2.5 flex w-80 justify-between items-center gap-2 text-sm font-medium border-1 border-gray-200 rounded-lg shadow-sm cursor-pointer select-none"
        >
            <Image
                width={22}
                height={16}
                alt="Flag"
                src={`http://purecatamphetamine.github.io/country-flag-icons/3x2/${localeToAlpha2CountryCode[locale || '']}.svg`}
            />
            <span className="flex-grow">
                {localeToLanguage[locale || 'en']}
            </span>
            <FiChevronDown
                className={`${isOptionstVisible && 'rotate-180'}`}
                stroke="#667085"
                size="20px"
            />
            {isOptionstVisible &&
                <ul
                    onClick={stopPropagation}
                    className="absolute left-0 top-12 w-full max-h-48 overflow-y-auto bg-white border-1 border-gray-200 rounded-lg shadow-sm z-10"
                >
                    {locales?.map(language =>
                        <Link
                            className="flex items-center gap-2 p-2.5 transition-all hover:bg-slate-200 cursor-pointer"
                            href={window.location.pathname.replace(locale || '', '')}
                            locale={language}
                            key={language}
                            onClick={toggleSelect}
                        >
                            <Image
                                width={22}
                                height={16}
                                alt={language}
                                src={`http://purecatamphetamine.github.io/country-flag-icons/3x2/${localeToAlpha2CountryCode[language]}.svg`}
                            />
                            {localeToLanguage[language]}
                        </Link>
                    )}
                </ul>
            }
        </div>
    );
};
