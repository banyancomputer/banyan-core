import Link from 'next/link';
import React, { ReactElement, useEffect, useRef, useState } from 'react';
import Image from 'next/image';
import { signOut, useSession } from 'next-auth/react';
import { useIntl } from 'react-intl';
import { useRouter } from 'next/router';
import { AiOutlineQuestionCircle } from 'react-icons/ai';
import { GrMailOption } from 'react-icons/gr';
import { MdOutlineVpnKey } from 'react-icons/md';
import { FiLogOut, FiSettings } from 'react-icons/fi';

import { popupClickHandler } from '@/utils';
import { useKeystore } from '@/contexts/keystore';
import { Action } from '../FileActions';

import { Input } from '../Input';

import { Logo } from '@static/images/common';

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

    const logout = () => {
        signOut();
        purgeKeystore();
    };

    const goTo = (path: string) => {
        return function () {
            router.push(path);
        }
    };

    const options = [
        new Action(`${messages.settings}`, <FiSettings size="20px" />, goTo('/settings')),
        new Action(`${messages.manageKeys}`, <MdOutlineVpnKey size="20px" />, goTo('/key-management')),
        new Action(`${messages.logout}`, <FiLogOut size="20px" />, logout)
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
        <header className="flex items-center justify-between border-b-2 border-c p-4">
            <Link href="/" className="font-semibold text-m flex-grow" >
                <Logo />
            </Link>
            <Input />
            <div className="flex flex-grow items-center justify-end gap-6">
                <div
                    className='relative w-10 h-10 flex items-center justify-center transition-all rounded-lg cursor-pointer hover:bg-slate-200'
                    ref={faqRef}
                    onClick={toggleFaqOptionsVisibility}
                >
                    <AiOutlineQuestionCircle size="20px" fill="#4A5578" />
                    {areFaqOpionsVisible &&
                        <div
                            className="absolute right-0 top-full w-36 flex flex-col items-stretch shadow-xl rounded-xl text-xs font-semibold overflow-hidden  bg-white cursor-pointer"
                        >
                            <a
                                className="flex items-center gap-2 py-select px-3 transition-all hover:bg-slate-100"
                                href='https://banyan8674.zendesk.com/hc/en-us'
                                target='_blank'
                            >
                                <MdOutlineVpnKey />
                                FAQ
                            </a>
                            <a
                                href='mailto:support@banyan8674.zendesk.com'
                                className="flex items-center gap-2 py-select px-3 transition-all hover:bg-slate-100"
                                target='_blank'
                            >
                                <GrMailOption />
                                {`${messages.contactUs}`}
                            </a>
                        </div>
                    }
                </div>
                <div
                    className="relative w-10 h-10 border-2 rounded-full cursor-pointer "
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
                            className="absolute right-0 top-full w-36 flex flex-col items-stretch shadow-xl rounded-xl text-xs font-semibold overflow-hidden  bg-white cursor-pointer"
                        >
                            {options.map(option =>
                                <div
                                    key={option.label}
                                    className="flex items-center gap-2 py-select px-3 transition-all hover:bg-slate-100"
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
