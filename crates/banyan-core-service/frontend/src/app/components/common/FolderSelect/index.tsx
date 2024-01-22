import React, { useEffect, useRef, useState } from 'react';
import { useIntl } from 'react-intl';

import { AddNewOption } from '../Select/AddNewOption';
import { CreateFolderModal } from '../Modal/CreateFolderModal ';
import { UploadFileModal } from '../Modal/UploadFileModal';

import { popupClickHandler } from '@/app/utils';
import { useModal } from '@/app/contexts/modals';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { useTomb } from '@/app/contexts/tomb';

import { ChevronUp } from '@static/images/common';

export interface FolderSelectProps {
    onChange: (option: string[]) => void;
    selectedBucket: Bucket;
    path: string[];
    onFolderCreation?: () => void;
};

export const FolderSelect: React.FC<FolderSelectProps> = ({ onChange, selectedBucket, onFolderCreation, path }) => {
    const { buckets, getSelectedBucketFolders } = useTomb();
    const selectRef = useRef<HTMLDivElement | null>(null);
    const [isOptionstVisible, setIsOptionsVisible] = useState(false);
    const [folder, setFolder] = useState(path);
    const [folders, setFolders] = useState<BrowserObject[]>([]);
    const { openModal } = useModal();
    const { messages } = useIntl();

    const toggleSelect = () => {
        setIsOptionsVisible(prev => !prev);
    };

    const handleSelect = (option: string[]) => {
        onChange(option);
        setFolder(option);
        setIsOptionsVisible(false);
    };

    const stopPropagation = (event: React.MouseEvent<HTMLUListElement>) => {
        event.stopPropagation();
    };

    const goAbove = () => {
        setFolder(prev => prev.slice(0, -1));
        handleSelect(folder.slice(0, -1));
    };

    const addNewFolder = () => {
        const action = onFolderCreation || (() => openModal(<UploadFileModal bucket={selectedBucket} path={folder} />));
        openModal(<CreateFolderModal
            path={folder}
            bucket={selectedBucket!}
        />
            ,
            action
        );
    };

    useEffect(() => {
        (async () => {
            const bucket = selectedBucket;
            if (!bucket.mount) return;

            try {
                const files = await getSelectedBucketFolders(bucket.id, folder);
                setFolders(files);
            } catch (error: any) { }
        })();
    }, [folder, buckets]);

    useEffect(() => {
        handleSelect(path);
    }, [path]);

    useEffect(() => {
        const listener = popupClickHandler(selectRef.current!, setIsOptionsVisible);
        document.addEventListener('click', listener);

        return () => document.removeEventListener('click', listener);
    }, [selectRef]);

    return (
        <div
            ref={selectRef}
            onClick={toggleSelect}
            className="relative p-2.5 flex justify-between items-center text-sm font-medium border-1 border-border-darken rounded-lg shadow-sm cursor-pointer select-none"
        >
            <span className="overflow-hidden text-ellipsis">
                /{folder.join('/')}
            </span>
            <span className={`${isOptionstVisible ? 'rotate-0' : 'rotate-180'}`}>
                <ChevronUp />
            </span>
            {isOptionstVisible &&
                <ul
                    onClick={stopPropagation}
                    className="absolute left-0 top-full w-full mt-2 max-h-48 overflow-y-auto bg-secondaryBackground border-1 border-border-darken rounded-lg shadow-sm z-10"
                >
                    {
                        folder.length ? <li
                            className="flex justify-between items-center p-2.5 transition-all hover:bg-bucket-bucketHoverBackground cursor-pointer"
                            onClick={goAbove}
                        >
                            ...
                        </li>
                            :
                            null
                    }
                    <AddNewOption label={`${messages.createFolder}`} action={addNewFolder} />
                    {folders.map((folderItem, index) =>
                        <li
                            className="flex justify-between items-center p-2.5 transition-all hover:bg-bucket-bucketHoverBackground cursor-pointer"
                            key={index}
                            onClick={() => handleSelect([...folder, folderItem.name])}
                        >
                            {folderItem.name}
                        </li>
                    )}
                </ul>
            }
        </div>
    );
};
