import React, { useEffect, useRef, useState } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/router';
import { useIntl } from 'react-intl';
import { signOut } from 'next-auth/react';

import { CreateBucketModal } from '../Modal/CreateBucketModal';

import { useTomb } from '@/contexts/tomb';
import { useModal } from '@/contexts/modals';
import { useKeystore } from '@/contexts/keystore';
import { popupClickHandler } from '@/utils';

import { ChevronUp, Home, Info, Logo, Logout, Mail, Plus, Question, Trash } from '@static/images/common';

export const Navigation = () => {
    const router = useRouter();
    const bucketId = router.query.id;
    const { buckets, trash } = useTomb();
    const { purgeKeystore } = useKeystore();
    const [isBucketsVisible, setIsBucketsVisible] = useState(false);
    const [areHelpOpionsVisible, setAreHelpOpionsVisible] = useState(false);
    const { messages } = useIntl();
    const { openModal } = useModal();
    const helpRef = useRef<HTMLDivElement | null>(null);

    const toggleBucketsVisibility = (event: React.MouseEvent<HTMLDivElement>) => {
        event.stopPropagation();
        event.preventDefault();
        setIsBucketsVisible(prev => !prev);
    };


    const toggleHelpOptionsVisibility = (event: any) => {
        setAreHelpOpionsVisible(prev => !prev);
    };

    const createBucket = () => {
        openModal(<CreateBucketModal />);
    };

    const logout = async () => {
        await signOut();
        await purgeKeystore();
    };

    useEffect(() => {
        if (isBucketsVisible) { return; }

        buckets.length && setIsBucketsVisible(true);
    }, [buckets]);

    useEffect(() => {
        const listener = popupClickHandler(helpRef.current!, setAreHelpOpionsVisible);
        document.addEventListener('click', listener);

        return () => {
            document.removeEventListener('click', listener);
        };
    }, [helpRef]);

    return (
        <nav className="flex flex-col w-navbar min-w-navbar bg-navigation-primary py-8 px-4 text-navigation-text border-r-2 border-r-navigation-border">
            <Link href="/" className="mb-7 flex text-xs" >
                <Logo width="174px" height="36px" />
            </Link>
            <div className="flex-grow py-8 border-t-2 border-b-2 border-navigation-separator text-navigation-text">
                <Link
                    href={'/'}
                    className={'flex items-center justify-between gap-3 py-2.5 px-3 w-full h-10  cursor-pointer rounded-md bg-navigation-secondary'}
                >
                    <Home />
                    <span className="flex-grow">
                        {`${messages.myDrives}`}
                    </span>
                    <span
                        onClick={toggleBucketsVisibility}
                        className={`${!isBucketsVisible && 'rotate-180'} ${!buckets.length && 'hidden'}`}
                    >
                        <ChevronUp />
                    </span>
                </Link>
                {
                    isBucketsVisible &&
                    <ul className="mt-3 mb-3 flex-col gap-2 px-4 text-xxs">
                        {
                            buckets.map(bucket =>
                                <li key={bucket.id}>
                                    <Link
                                        href={`/bucket/${bucket.id}`}
                                        className={'relative flex items-center justify-between gap-2  w-full h-10  cursor-pointer '}
                                    >
                                        <span className="absolute w-4 h-11 bottom-1/2 border-2 border-transparent border-l-navigation-secondary border-b-navigation-secondary">
                                        </span>
                                        <span className={`ml-5 py-2 px-2 flex-grow whitespace-nowrap overflow-hidden rounded-md verflow-ellipsis ${bucketId === bucket.id && 'bg-navigation-secondary'}`}>
                                            {bucket.name}
                                        </span>
                                    </Link>
                                </li>
                            )
                        }
                    </ul>
                }
                {/* <Link
                    href="/trash"
                    className={`flex items-center justify-between  gap-2 py-2 px-3 w-full h-10 cursor-pointer rounded-md text-xs ${router.pathname === '/trash' && 'bg-navigation-secondary'}`}
                >
                    <Trash />
                    <span className="flex-grow">
                        {`${messages.trash}`}
                    </span>
                    <span className={`px-2 py-1 bg-navigation-text text-navigation-secondary rounded-full text-xxs ${!trash.files.length && 'hidden'}`}>
                        {trash.files.length}
                    </span>
                </Link> */}
                <button
                    onClick={createBucket}
                    className="mt-2 flex items-center gap-3 py-2 px-3 text-navigation-textSecondary"
                >
                    <Plus />
                    {`${messages.newDrive}`}
                </button>
            </div>
            <div className="flex flex-col gap-2 mt-6 pl-2 pt-3 pr-8 text-navigation-textSecondary text-xs">
                <span
                    className="relative flex items-center gap-3 py-2.5 cursor-pointer"
                    onClick={toggleHelpOptionsVisibility}
                    ref={helpRef}
                >
                    <Info />
                    {`${messages.help}`}
                    {areHelpOpionsVisible &&
                        <div
                            className="absolute left-0 top-10 w-full flex flex-col items-stretch shadow-xl rounded-xl overflow-hidden text-xs font-semibold overflow-hiddenaa bg-bucket-actionsBackground cursor-pointer text-bucket-actionsText"
                        >
                            <a
                                className="flex items-center gap-2 py-2.5 px-3 transition-all hover:bg-hover"
                                href="https://banyan8674.zendesk.com/hc/en-us"
                                target="_blank"
                            >
                                <span className="text-button-primary">
                                    <Question />
                                </span>
                                FAQ
                            </a>
                            <a
                                href="mailto:support@banyan8674.zendesk.com"
                                className="flex items-center gap-2 py-2.5 px-3 transition-all hover:bg-hover"
                                target="_blank"
                            >
                                <span className="text-button-primary">
                                    <Mail />
                                </span>
                                {`${messages.contactUs}`}
                            </a>
                        </div>
                    }
                </span>
                <span
                    className="flex items-center gap-3 py-2.5 cursor-pointer"
                    onClick={logout}
                >
                    <Logout />
                    <span className="text-navigation-text">
                        {`${messages.logoutAccount}`}
                    </span>
                </span>
            </div>
        </nav>
    );
};

