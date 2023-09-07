import React, { ReactElement, useEffect, useState } from 'react';
import { useIntl } from 'react-intl';
import Link from 'next/link';
import { IoMdClose } from 'react-icons/io';
import { FiSearch } from 'react-icons/fi';
import { useRouter } from 'next/router';

import { useTomb } from '@/contexts/tomb';
import { useFolderLocation } from '@/hooks/useFolderLocation';

import { FileIcon } from '../FileIcon';

interface SeatchOption {
    label: string;
    path: string;
};

export const Input = React.memo(() => {
    const { buckets, selectedBucket } = useTomb();
    const [search, setSearch] = useState('');
    const [searchList, setSearchList] = useState<SeatchOption[]>([]);
    const { messages } = useIntl();
    const { pathname } = useRouter();
    const folderLocation = useFolderLocation();

    const clearSearch = () => {
        setSearch('');
    };

    /** Ceates array of single-level elements to be able to go through them by search */
    useEffect(() => {
        if (pathname === '/bucket/[id]') {
            setSearchList(
                selectedBucket ? [...selectedBucket.files?.map(file => ({ label: file.name, path: `/bucket/${selectedBucket?.id}?${folderLocation.join('/')}${file.type === 'dir' ? `${folderLocation.length ? `/${file.name}` : file.name}` : ''}` })),
                { label: selectedBucket?.name, path: `/bucket/${selectedBucket?.id}` }
                ] : []
            )
            return;
        };

        setSearchList(buckets.map(bucket =>
            [...bucket?.files?.map(file => ({ label: file.name, path: `/bucket/${bucket?.id}?${file.type === 'dir' ? `${file.name}` : ''}` })),
            { label: bucket.name, path: `/bucket/${bucket.id}` }]
        ).flat()
        );
    }, [buckets, selectedBucket, pathname]);

    return (
        <div className="flex relative flex-grow max-w-xl">
            <span className="absolute left-4 top-1/2 -translate-y-1/2">
                <FiSearch size="20px" stroke="#667085" />
            </span>
            <input
                className={`input w-full h-10 py-3 px-4 rounded-xl border-gray-400  pl-12 focus:outline-none`}
                value={search}
                onChange={event => setSearch(event.target.value)}
                placeholder={`${messages.search}`}
            />
            {search &&
                <>
                    <span
                        className="absolute right-4 top-1/2 -translate-y-1/2 text-gray-500 bg-gray-300 h-fit rounded-full cursor-pointer"
                        onClick={clearSearch}
                    >
                        <IoMdClose size="20px" />
                    </span>
                    <div
                        className="absolute top-11 left-0 w-full max-h-48 flex flex-col items-stretch z-10 bg-white rounded-lg shadow-md overflow-y-scroll"
                    >
                        {searchList.filter(element => element.label.includes(search)).map((element, index) =>
                            <Link
                                href={element.path}
                                className="flex items-center gap-2 py-2 px-3 transition-all hover:bg-slate-200"
                                key={index}
                                onClick={clearSearch}
                            >
                                <FileIcon fileName={element.label} />
                                {element.label}
                            </Link>
                        )}
                    </div>
                </>
            }
        </div>
    );
});

