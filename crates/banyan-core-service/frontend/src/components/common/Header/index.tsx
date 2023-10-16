import React, { useEffect, useRef, useState } from 'react';
import Image from 'next/image';
import { signOut, useSession } from 'next-auth/react';
import { useIntl } from 'react-intl';
import { useRouter } from 'next/router';

import { popupClickHandler } from '@/utils';
import { useKeystore } from '@/contexts/keystore';
import { Action } from '../FileActions';

import { Headphones, Key, LogoutAlternative, Mail, Question, Settings } from '@static/images/common';

export const Header = () => {
    const userControlsRef = useRef<HTMLDivElement | null>(null);
    const faqRef = useRef<HTMLDivElement | null>(null);
    const { purgeKeystore } = useKeystore();
    const { messages } = useIntl();
    const router = useRouter();
    const { data: session } = useSession();
    const [areProfileOptionsVisible, setAreProfileOptionsVisible] = useState(false);
    const [areFaqOpionsVisible, setAreFaqOpionsVisible] = useState(false);

    const toggleProfileOptionsVisibility = () => {
        setAreProfileOptionsVisible(prev => !prev);
    };

    const toggleFaqOptionsVisibility = (event: any) => {
        setAreFaqOpionsVisible(prev => !prev);
    };

    const logout = async () => {
        await signOut();
        await purgeKeystore();
    };

    const goTo = (path: string) => {
        return function () {
            router.push(path);
        }
    };

    const options = [
        new Action(`${messages.settings}`, <Settings/>, goTo('/account/settings')),
        new Action(`${messages.manageKeys}`, <Key  />, goTo('/account/manage-keys')),
        new Action(`${messages.logout}`, <LogoutAlternative />, logout)
    ];

    useEffect(() => {
        const listener = popupClickHandler(userControlsRef.current!, setAreProfileOptionsVisible);
        document.addEventListener('click', listener);

        return () => {
            document.removeEventListener('click', listener);
        };
    }, [userControlsRef]);

    useEffect(() => {
        const listener = popupClickHandler(faqRef.current!, setAreFaqOpionsVisible);
        document.addEventListener('click', listener);

        return () => {
            document.removeEventListener('click', listener);
        };
    }, [faqRef]);

    return (
        <header className="flex items-center justify-between border-b-2 border-table-border p-4 bg-navigation-primary">
            {/* <SearchInput /> */}
            <div className="flex flex-grow items-center justify-end gap-6">
                <div
                    className='relative w-10 h-10 flex items-center justify-center transition-all rounded-lg cursor-pointer hover:bg-navigation-secondary text-navigation-text'
                    ref={faqRef}
                    onClick={toggleFaqOptionsVisibility}
                >
                    <Headphones />
                    {areFaqOpionsVisible &&
                        <div
                            className="absolute right-0 top-12 w-36 flex flex-col items-stretch shadow-xl rounded-xl text-xs font-semibold overflow-hidden  bg-mainBackground cursor-pointer text-gray-900"
                        >
                            <a
                                className="flex items-center gap-2 py-2.5 px-3 transition-all hover:bg-hover"
                                href='https://banyan8674.zendesk.com/hc/en-us'
                                target='_blank'
                            >
                                <Question  />
                                FAQ
                            </a>
                            <a
                                href='mailto:support@banyan8674.zendesk.com'
                                className="flex items-center gap-2 py-2.5 px-3 transition-all hover:bg-hover"
                                target='_blank'
                            >
                                <Mail />
                                {`${messages.contactUs}`}
                            </a>
                        </div>
                    }
                </div>
                <div
                    className="relative w-10 h-10 rounded-full cursor-pointer "
                    onClick={toggleProfileOptionsVisibility}
                    ref={userControlsRef}
                >
                    {session?.user?.image ?
                        <Image
                            className="rounded-full"
                            src={session?.user?.image}
                            width={40}
                            height={40}
                            alt="User Avatar"
                        />
                        :
                        null
                    }
                    {areProfileOptionsVisible &&
                        <div
                            className="absolute z-10 right-0 top-12 flex flex-col items-stretch shadow-xl rounded-xl text-xs font-semibold overflow-hidden  bg-mainBackground cursor-pointer"
                        >
                            {options.map(option =>
                                <div
                                    key={option.label}
                                    className="flex items-center gap-2 py-2.5 px-3 whitespace-nowrap transition-all hover:bg-hover"
                                    onClick={option.value}
                                >
                                    {option.icon}
                                    {option.label}
                                </div>
                            )}
                        </div>
                    }
                </div>
            </div>
        </header >
    );
};
