import React, { useState } from 'react';
import Link from 'next/link';

import { MockBucket } from '@/lib/interfaces/bucket';

import { ArrowDown, Cross, Directory, Folder } from '@static/images/common';

export const Navigation = () => {
    const [isBucketsVisible, setIsBucketsVisible] = useState(false);
    const [isStorageBlockVisible, setIsStorageBlockVisible] = useState(true);

    const toggleBucketsVisibility = () => {
        setIsBucketsVisible(prev => !prev);
    };
    const toggleStorageVisibility = () => {
        setIsStorageBlockVisible(prev => !prev);
    };

    /** TODO: delete after  api connection. */
    const MOCK_BUCKETS = [
        new MockBucket('', 'Test1', ''),
        new MockBucket('', 'Test2', ''),
        new MockBucket('', 'Test3', ''),
    ];

    return (
        <nav className="flex flex-col w-navbar bg-navigation-primary py-8 px-4 text-navigation-text border-r-2 border-r-navigation-border font-bold">
            <div className="flex-grow">
                <div
                    onClick={toggleBucketsVisibility}
                    className="flex items-center justify-between gap-2 py-2 px-3 w-full h-10 rounded-md bg-navigation-secondary cursor-pointer"
                >
                    <Directory />
                    <span className="flex-grow">
                        My Buckets
                    </span>
                    <span className="px-2 py-1 bg-navigation-text text-navigation-secondary rounded-full text-xxs font-medium">
                        1,189
                    </span>
                    <span>
                        <ArrowDown />
                    </span>
                </div>
                {
                    isBucketsVisible &&
                    <ul className="mt-3 mb-3 flex-col gap-2 px-4">
                        {
                            MOCK_BUCKETS.map((bucket, index) =>
                                <li>
                                    <Link
                                        href={`/${bucket.id}`}
                                        className="flex items-center justify-between gap-2 py-2 px-3 w-full h-10  cursor-pointer"
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
                    href="/"
                    className="flex items-center justify-between  gap-2 py-2 px-3 w-full h-10 cursor-pointer"
                >
                    <Folder />
                    <span className="flex-grow">
                        Trash
                    </span>
                    <span className="px-2 py-1 bg-navigation-secondary text-navigation-textSecondary rounded-full text-xxs font-normal">
                        54
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
                    <span className="text-xs font-normal">You’ve used 18 out of 20 TB.</span>
                </div>
            }
            <div className="flex flex-col mt-6 pl-2 pt-3 pr-8 border-t-2 border-gray-200">
                <span>Banyan Computer</span>
                <span className="font-normal">Decentralized Storage</span>
            </div>
        </nav>
    );
};

