import React, { useEffect, useState } from 'react';
import Link from 'next/link';
import { useTomb } from '@/contexts/tomb';
import { useSearchParams } from 'next/navigation';
import { useRouter } from 'next/router';

import { ArrowDown, Cross, Directory, Folder } from '@static/images/common';
import { convertFileSize } from '@/utils/storage';

export const Navigation = () => {
    /** TODO: replace by data from api. */
    const MOCK_STORAGE_LIMIT = 2e+13;

    const searchParams = useSearchParams();
    const router = useRouter();
    const bucketId = searchParams.get('id');
    const { buckets, trash, usedStorage } = useTomb();
    const [isBucketsVisible, setIsBucketsVisible] = useState(false);
    const [isStorageBlockVisible, setIsStorageBlockVisible] = useState(true);

    const toggleBucketsVisibility = (event: React.MouseEvent<HTMLDivElement>) => {
        event.stopPropagation();
        event.preventDefault();
        setIsBucketsVisible(prev => !prev);
    };

    const toggleStorageVisibility = () => {
        setIsStorageBlockVisible(prev => !prev);
    };

    return (
        <nav className="flex flex-col w-navbar bg-navigation-primary py-8 px-4 text-navigation-text border-r-2 border-r-navigation-border font-bold">
            <div className="flex-grow">
                <Link
                    href={'/'}
                    className={`flex items-center justify-between gap-2 py-2 px-3 w-full h-10  cursor-pointer rounded-md ${router.pathname === '/' && 'bg-navigation-secondary'}`}
                >
                    <Directory />
                    <span className="flex-grow">
                        My Buckets
                    </span>
                    <span className="px-2 py-1 bg-navigation-text text-navigation-secondary rounded-full text-xxs font-medium">
                        {buckets.map(bucket => bucket.files.length).reduce((accumulator, currentValue) => accumulator + currentValue, 0)}
                    </span>
                    <span
                        onClick={toggleBucketsVisibility}
                        className={`${isBucketsVisible && 'rotate-180'} `}
                    >
                        <ArrowDown />
                    </span>
                </Link>
                {
                    isBucketsVisible &&
                    <ul className="mt-3 mb-3 flex-col gap-2 px-4">
                        {
                            buckets.map(bucket =>
                                <li key={bucket.id}>
                                    <Link
                                        href={`/bucket/${bucket.id}`}
                                        className={`flex items-center justify-between gap-2 py-2 px-3 w-full h-10  cursor-pointer rounded-md ${bucketId === bucket.id && 'bg-navigation-secondary'}`}
                                    >
                                        <Directory />
                                        <span className="flex-grow">
                                            {bucket.name}
                                        </span>
                                    </Link>
                                </li>
                            )
                        }
                    </ul>
                }
                <Link
                    href="/trash"
                    className={`flex items-center justify-between  gap-2 py-2 px-3 w-full h-10 cursor-pointer rounded-md ${router.pathname === '/trash' && 'bg-navigation-secondary'}`}
                >
                    <Folder />
                    <span className="flex-grow">
                        Trash
                    </span>
                    <span className="px-2 py-1 bg-navigation-text text-navigation-secondary rounded-full text-xxs font-normal">
                        {trash.files.length}
                    </span>
                </Link>
            </div>
            {isStorageBlockVisible &&
                <div className="bg-white rounded-lg p-4">
                    <span className="flex justify-between items-center ">
                        Storage
                        <button onClick={toggleStorageVisibility}>
                            <Cross />
                        </button>
                    </span>
                    <span className="text-xs font-normal">You’ve used <span className="uppercase">{convertFileSize(usedStorage)}</span> out of 20 TB.</span>
                    <progress className="progress w-56" value={usedStorage} max={MOCK_STORAGE_LIMIT}></progress>
                </div>
            }
            <div className="flex flex-col mt-6 pl-2 pt-3 pr-8 border-t-2 border-gray-200">
                <span>Banyan Computer</span>
                <span className="font-normal">Decentralized Storage</span>
            </div>
        </nav>
    );
};

