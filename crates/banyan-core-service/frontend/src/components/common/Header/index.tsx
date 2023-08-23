import Link from 'next/link';
import React, { useEffect, useRef, useState } from 'react';
import Image from 'next/image';
import { signOut, useSession } from 'next-auth/react';
import { useIntl } from 'react-intl';
import { useRouter } from 'next/router';
import { AiOutlineQuestionCircle } from "react-icons/ai"
import { FiSettings, FiSearch } from "react-icons/fi"

import { popupClickHandler } from '@/utils';

import { Input } from '../Input';

export const Header = () => {
    const userControlsRef = useRef<HTMLDivElement | null>(null);
    const languagesControlsRef = useRef<HTMLDivElement | null>(null);
    const { messages } = useIntl();
    const { locales, locale } = useRouter();
    const { data } = useSession();
    const [isLogoutButtonVisible, setIsLogoutButtonVisible] = useState(false);
    const [isLanguageControlsVisible, setIsLanguageControlsVisible] = useState(false);

    const toggleLogoutVisibility = () => {
        setIsLogoutButtonVisible(prev => !prev);
    };
    const toggleLanguageVisibility = () => {
        setIsLanguageControlsVisible(prev => !prev);
    };

    useEffect(() => {
        const listener = popupClickHandler(userControlsRef.current!, setIsLogoutButtonVisible);
        document.addEventListener('click', listener);

        return () => {
            document.removeEventListener('click', listener);
        };
    }, [userControlsRef]);

    useEffect(() => {
        const listener = popupClickHandler(languagesControlsRef.current!, setIsLanguageControlsVisible);
        document.addEventListener('click', listener);

        return () => {
            document.removeEventListener('click', listener);
        };
    }, [languagesControlsRef]);

    return (
        <header className="flex items-center justify-between border-b-2 border-c p-4">
            <Link href="/" className="font-semibold text-m flex-grow" >
                Banyan Computer
            </Link>
            <Input
                placeholder={`${messages.search}`}
                icon={<FiSearch size="20px" stroke="#4A5578" />}
            />
            <div className="flex flex-grow items-center justify-end gap-6">
                <Link href="/key-management" className="font-semibold text-nav mr-4" >
                    {`${messages.manageKeyAccess}`}
                </Link>
                <div
                    className='relative cursor-pointer'
                    onClick={toggleLanguageVisibility}
                    ref={languagesControlsRef}
                >
                    <FiSettings size="20px" stroke="#4A5578" />
                    {isLanguageControlsVisible &&
                        <div
                            className='absolute top-full left-1/2 -translate-x-1/2 flex flex-col gap-1  rounded-xl bg-white shadow-xld overflow-hidden'
                        >{
                                locales?.map(language =>
                                    <Link
                                        key={language}
                                        href={window.location.pathname.replace(locale || '', '')}
                                        locale={language}
                                        className='p-2 hover:bg-slate-100'
                                    >
                                        {language}
                                    </Link>
                                )}
                        </div>
                    }
                </div>
                <Link href="/faq">
                    <AiOutlineQuestionCircle size="20px" fill="#4A5578" />
                </Link>
                <div
                    className="relative w-10 h-10 border-2 rounded-full cursor-pointer "
                    onClick={toggleLogoutVisibility}
                    ref={userControlsRef}
                >
                    {data?.user?.image ?
                        <Image
                            className="rounded-full"
                            src={data?.user?.image}
                            width={40}
                            height={40}
                            alt="User Avatar"
                        />
                        :
                        null
                    }
                    {isLogoutButtonVisible &&
                        <div
                            className="absolute right-0 -bottom-12 w-36 h-10 flex items-center shadow-xl p-2 rounded-xl text-xs font-semibold  bg-white cursor-pointer hover:bg-slate-100"
                            onClick={() => signOut()}
                        >
                            {`${messages.logout}`}
                        </div>
                    }
                </div>
            </div>
        </header >
    );
};
