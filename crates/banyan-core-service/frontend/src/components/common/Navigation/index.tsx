import React, { useEffect, useState } from 'react';
import Link from 'next/link';
import { useSearchParams } from 'next/navigation';
import { useRouter } from 'next/router';
import { FiChevronDown, FiTrash2 } from 'react-icons/fi';
import { IoIosAdd, IoMdClose } from 'react-icons/io';
import { useIntl } from 'react-intl';

import { CreateBucketModal } from '../Modal/CreateBucketModal';

import { useTomb } from '@/contexts/tomb';
import { convertFileSize } from '@/utils/storage';
import { useModal } from '@/contexts/modals';

import { Directory } from '@static/images/common';

export const Navigation = () => {
    const searchParams = useSearchParams();
    const router = useRouter();
    const bucketId = searchParams.get('id');
    const { buckets, trash, usedStorage, usageLimit } = useTomb();
    const [isBucketsVisible, setIsBucketsVisible] = useState(false);
    const [isStorageBlockVisible, setIsStorageBlockVisible] = useState(true);
    const { messages } = useIntl();
    const { openModal } = useModal()
    const toggleBucketsVisibility = (event: React.MouseEvent<HTMLDivElement>) => {
        event.stopPropagation();
        event.preventDefault();
        setIsBucketsVisible(prev => !prev);
    };

    const toggleStorageVisibility = () => {
        setIsStorageBlockVisible(prev => !prev);
    };

    const createBucket = () => {
        openModal(<CreateBucketModal />);
    };

    useEffect(() => {
        if (isBucketsVisible) return;

        buckets.length && setIsBucketsVisible(true);
    }, [buckets])

    return (
        <nav className="flex flex-col w-navbar min-w-navbar bg-navigation-primary py-8 px-4 text-navigation-text border-r-2 border-r-navigation-border font-semibold">
            <div className="flex-grow">
                <Link
                    href={'/'}
                    className={`flex items-center justify-between gap-2 py-2 px-3 w-full h-10  cursor-pointer rounded-md ${router.pathname === '/' && 'bg-navigation-secondary'}`}
                >
                    <Directory />
                    <span className="flex-grow">
                        {`${messages.myBuckets}`}
                    </span>
                    <span className={`px-2 py-1 bg-navigation-text text-navigation-secondary rounded-full text-xxs font-medium ${!buckets.length && 'hidden'}`}>
                        {buckets.map(bucket => bucket.files.length).reduce((accumulator, currentValue) => accumulator + currentValue, 0)}
                    </span>
                    <span
                        onClick={toggleBucketsVisibility}
                        className={`${isBucketsVisible && 'rotate-180'} ${!buckets.length && 'hidden'} `}
                    >
                        <FiChevronDown size="20px" stroke="#5D6B98" />
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
                    className={`flex items-center justify-between  gap-2 py-2 px-3 w-full h-10 cursor-pointer rounded-md ${router.pathname === '/trash' && 'bg-navigation-secondary'} hidden`}
                >
                    <FiTrash2 size="24px" stroke="#5e6c97" />
                    <span className="flex-grow">
                        {`${messages.trash}`}
                    </span>
                    <span className={`px-2 py-1 bg-navigation-text text-navigation-secondary rounded-full text-xxs font-normal ${!trash.files.length && 'hidden'}`}>
                        {trash.files.length}
                    </span>
                </Link>
                <button
                    onClick={createBucket}
                    className="mt-2 flex items-center gap-3 py-2 px-3"
                >
                    <IoIosAdd size="24px" fill="#5D6B98" />
                    {`${messages.newBucket}`}
                </button>
            </div>
            {isStorageBlockVisible &&
                <div className="bg-white rounded-lg p-4">
                    <span className="flex justify-between items-center ">
                        {`${messages.storage}`}
                        <button onClick={toggleStorageVisibility}>
                            <IoMdClose size="20px" />
                        </button>
                    </span>
                    <span className="text-xs font-normal">{` ${messages.youHaveUsed} `}
                        <span className="uppercase">{convertFileSize(usedStorage)}</span>
                        {` ${messages.outOf} `}
                        <span className="uppercase">{convertFileSize(usageLimit)}</span>.
                    </span>
                    <progress className="progress w-full" value={usedStorage} max={usageLimit}></progress>
                </div>
            }
            <div className="flex flex-col mt-6 pl-2 pt-3 pr-8 border-t-2 border-gray-200 text-gray-600">
                <span>Banyan Computer</span>
                <span className="font-normal">{`${messages.decentralizedStorage}`}</span>
            </div>
        </nav>
    );
};

