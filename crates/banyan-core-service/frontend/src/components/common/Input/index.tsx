import React, { ReactElement, useEffect, useState } from 'react';
import { useTomb } from '@/contexts/tomb';
import { IoMdClose } from "react-icons/io"
import Link from 'next/link';
import { FileIcon } from '../FileIcon';

interface SeatchOption {
    label: string;
    path: string;
}

export const Input: React.FC<{
    placeholder: string;
    icon?: ReactElement;
}> = ({
    placeholder,
    icon,
}) => {
        const { buckets } = useTomb();
        const [search, setSearch] = useState('');
        const [searchList, setSearchList] = useState<Array<SeatchOption>>([]);

        const clearSearch = () => {
            setSearch('');
        };

        /** Ceates array of single-level elements to be able to go through them by search */
        useEffect(() => {
            setSearchList(buckets.map(buclet =>
                [...buclet.files.map(file => ({ label: file.name, path: `/bucket/${buclet.id}` })),
                { label: buclet.name, path: `/bucket/${buclet.id}` }]
            ).flat()
            );
        }, [buckets]);

        return (
            <div className="flex relative flex-grow max-w-xl">
                <span className="absolute left-4 top-1/2 -translate-y-1/2">
                    {icon}
                </span>
                <input
                    className={`input w-full h-10 py-3 px-4 rounded-xl border-gray-400  ${icon ? 'pl-12' : ''} focus:outline-none`}
                    value={search}
                    onChange={event => setSearch(event.target.value)}
                    placeholder={placeholder}
                />
                {search &&
                    <>
                        <span
                            className='absolute right-4 top-1/2 -translate-y-1/2 text-gray-500 bg-gray-300 h-fit rounded-full cursor-pointer'
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
                                    className='flex items-center gap-2 py-2 px-3 transition-all hover:bg-slate-200'
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
        )
    }

