import Link from 'next/link';
import React, { useEffect, useRef, useState } from 'react';
import Image from 'next/image';
import { signOut, useSession } from 'next-auth/react';

import { otsideClickHandler } from '@/utils';

import { Input } from '../Input';

import { Question, Search, Settings } from '@static/images/common';

export const Header = () => {
    const controlsRef = useRef<HTMLDivElement | null>(null);
    const { data } = useSession();
    const [isLogoutButtonVisible, setIsLogoutButtonVisible] = useState(false);

    const toggleVisibility = () => {
        setIsLogoutButtonVisible(prev => !prev);
    };

    useEffect(() => {
        const listener = otsideClickHandler(controlsRef.current!, setIsLogoutButtonVisible);
        document.addEventListener('click', listener);

        return () => {
            document.removeEventListener('click', listener);
        };
    }, [controlsRef]);

    return (
        <header className="flex items-center justify-between border-b-2 border-c h-navbar px-4">
            <Link href="/" className="font-semibold text-m flex-grow" >
                Banyan Computer
            </Link>
            <Input
                placeholder="Search"
                icon={<Search />}
                onChange={() => { }}
            />
            <div className="flex flex-grow items-center justify-end gap-6">
                <Link href="/key-management" className="font-semibold text-nav mr-4" >
                    Manage Key Access
                </Link>
                <Link href="/">
                    <Settings />
                </Link>
                <Link href="/">
                    <Question />
                </Link>
                <div
                    className="relative w-10 h-10 border-2 rounded-full cursor-pointer "
                    onClick={toggleVisibility}
                    ref={controlsRef}
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
                            Log Out
                        </div>
                    }
                </div>
            </div>
        </header>
    );
};
