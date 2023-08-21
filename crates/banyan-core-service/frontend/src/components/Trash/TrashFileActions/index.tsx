import React, { ReactElement } from 'react';
import { useIntl } from 'react-intl';
import { FiTrash2 } from "react-icons/fi";
import { PiArrowsLeftRight } from "react-icons/pi";

import { useTomb } from '@/contexts/tomb';
import { Bucket, BucketFile } from '@/lib/interfaces/bucket';
import { useModal } from '@/contexts/modals';
import { MoveToModal } from '../../common/Modal/MoveToModal';
import { FileAction } from '../../Buckets/FileActions';


export const TrashFileActions: React.FC<{ bucket: Bucket, file: BucketFile }> = ({ bucket, file }) => {
    const { messages } = useIntl();
    const { } = useTomb();
    const { openModal } = useModal();

    const moveTo = () => {
        openModal(<MoveToModal file={file} />)
    };
    const makeCopy = async () => {
        try {

        } catch (error: any) { }

    };

    const remove = async () => {
        try {

        } catch (error: any) { }

    };

    const acrions = [
        new FileAction(`${messages.moveTo}`, <PiArrowsLeftRight size="18px" />, moveTo),
        new FileAction(`${messages.remove}`, <FiTrash2 size="18px" />, remove),
    ];

    return (
        <div className='relative w-48 text-xs font-medium bg-white rounded-xl shadow-md z-10 text-gray-900'>{
            acrions.map(action =>
                <div
                    key={action.label}
                    className='w-full flex items-center gap-2 py-2 px-3 border-b-1 border-gray-200 transition-all hover:bg-slate-200'
                    onClick={action.value}
                >
                    {action.icon} {action.label}
                </div>
            )
        }</div>
    )
}
