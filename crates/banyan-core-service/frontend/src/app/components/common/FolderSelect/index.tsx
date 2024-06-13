import React, { useEffect, useRef, useState } from 'react';

import { AddNewOption } from '@components/common/Select/AddNewOption';
import { CreateFolderModal } from '@components/common/Modal/CreateFolderModal ';
import { UploadFileModal } from '@components/common/Modal/UploadFileModal';

import { popupClickHandler } from '@/app/utils';
import { openModal } from '@store/modals/slice';
import { BrowserObject, Bucket } from '@/app/types/bucket';
import { useAppDispatch, useAppSelector } from '@store/index';

import { ChevronUp } from '@static/images/common';

export interface FolderSelectProps {
    onChange: (option: string[]) => void;
    selectedBucket: Bucket;
    onFolderCreation?: (path?: string[]) => void;
    selectedFolder: string[];
};

export const FolderSelect: React.FC<FolderSelectProps> = ({ onChange, selectedBucket, selectedFolder, onFolderCreation }) => {
    const dispatch = useAppDispatch();
    const selectRef = useRef<HTMLDivElement | null>(null);
    const [isOptionstVisible, setIsOptionsVisible] = useState(false);
    const [folder, setFolder] = useState(selectedFolder);
    const [folders, setFolders] = useState<BrowserObject[]>([]);
    const messages = useAppSelector(state => state.locales.messages.coponents.common.folderSelect);
    const { buckets } = useAppSelector(state => state.tomb);

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
        const action = onFolderCreation || (() => {
            dispatch(openModal({
                content: <UploadFileModal bucket={selectedBucket} path={folder} />,
                path: [selectedBucket.name, ...folder]
            }))
        });
        dispatch(openModal(
            {
                content: <CreateFolderModal
                    path={folder}
                    bucket={selectedBucket!}
                    onSuccess={(path: string[]) => action(path)}
                />,
                path: [selectedBucket.name, ...selectedFolder],
                onBack: () => action(folder)
            }
        ));
    };

    useEffect(() => {
        (async () => {
            const bucket = selectedBucket;
            if (!bucket.mount) return;
            const files = await bucket.mount.ls(folder);
            setFolders(files.filter(file => file.type === 'dir'));
        })();
    }, [folder, buckets]);

    useEffect(() => {
        handleSelect(selectedFolder);
    }, [selectedFolder]);

    useEffect(() => {
        const listener = popupClickHandler(selectRef.current!, setIsOptionsVisible);
        document.addEventListener('click', listener);

        return () => document.removeEventListener('click', listener);
    }, [selectRef]);

    return (
        <div
            ref={selectRef}
            onClick={toggleSelect}
            className="relative p-2.5 flex justify-between items-center text-sm font-medium border-1 border-border-darken rounded-md shadow-sm cursor-pointer select-none"
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
