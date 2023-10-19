import React, { ReactElement, useEffect, useState } from 'react';
import { useIntl } from 'react-intl';
import Link from 'next/link';
import { IoMdClose } from 'react-icons/io';
import { FiSearch } from 'react-icons/fi';
import { useRouter } from 'next/router';

import { useTomb } from '@/contexts/tomb';
import { useFolderLocation } from '@/hooks/useFolderLocation';
import { useFilePreview } from '@/contexts/filesPreview';
import { Bucket } from '@/lib/interfaces/bucket';

import { FileIcon } from '../FileIcon';

interface SearchOption {
    bucket: Bucket;
    label: string;
    path: string;
};

export const SearchInput = React.memo(() => {
    const { buckets, selectedBucket } = useTomb();
    const [search, setSearch] = useState('');
    const [searchList, setSearchList] = useState<SearchOption[]>([]);
    const { messages } = useIntl();
    const { pathname, push } = useRouter();
    const folderLocation = useFolderLocation();
    const { openFile } = useFilePreview();

    const clearSearch = () => {
        setSearch('');
    };
    const goTo = (path: string) => {
        push(path);
        clearSearch();
    };

    const previewFile = (bucket: Bucket, name: string) => {
        openFile(bucket, name, folderLocation);
        clearSearch();
    };

    /** Ceates array of single-level elements to be able to go through them by search */
    useEffect(() => {
        if (pathname === '/bucket/[id]') {
            setSearchList(
                selectedBucket ? [
                    ...selectedBucket.files?.map(file => ({ bucket: selectedBucket, label: file.name, path: file.type === 'dir' ? `/bucket/${selectedBucket?.id}?${folderLocation.join('/')}${folderLocation.length ? `/${file.name}` : file.name}` : '' })),
                    { bucket: selectedBucket, label: selectedBucket?.name, path: `/bucket/${selectedBucket?.id}` }
                ] : []
            )
            return;
        };

        setSearchList(buckets.map(bucket =>
            [...bucket?.files?.map(file => ({ bucket, label: file.name, path: file.type === 'dir' ? `/bucket/${bucket?.id}?${folderLocation.join('/')}${folderLocation.length ? `/${file.name}` : file.name}` : '' })),
            { bucket, label: bucket.name, path: `/bucket/${bucket.id}` }]
        ).flat()
        );
    }, [buckets, selectedBucket, pathname]);

    return (
        <div className="flex relative flex-grow max-w-xl">
            <span className="absolute left-4 top-1/2 -translate-y-1/2">
                <FiSearch size="20px" stroke="#667085" />
            </span>
            <input
                className={`input w-full h-10 py-3 px-4 rounded-xl bg-secondaryBackground border-border-darken  pl-12 focus:outline-none`}
                value={search}
                onChange={event => setSearch(event.target.value)}
                placeholder={`${messages.search}`}
            />
            {search &&
                <>
                    <span
                        className="absolute right-4 top-1/2 -translate-y-1/2 text-text-900 bg-border h-fit rounded-full cursor-pointer"
                        onClick={clearSearch}
                    >
                        <IoMdClose size="20px" />
                    </span>
                    <div
                        className="absolute top-11 left-0 w-full max-h-48 flex flex-col items-stretch z-10 bg-secondaryBackground rounded-lg shadow-md overflow-y-scroll"
                    >
                        {searchList.filter(element => element.label.toLocaleLowerCase().includes(search.toLocaleLowerCase())).map((element, index) =>
                            <div
                                className="flex items-center gap-2 py-2 px-3 transition-all cursor-pointer hover:bg-hover"
                                key={index}
                                onClick={() => element.path ? goTo(element.path) : previewFile(element.bucket, element.label)}
                            >
                                <FileIcon fileName={element.label} />
                                {element.label}
                            </div>
                        )}
                    </div>
                </>
            }
        </div>
    );
});

