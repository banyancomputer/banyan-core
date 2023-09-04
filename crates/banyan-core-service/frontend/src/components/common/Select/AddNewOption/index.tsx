import React from 'react';
import { IoMdAdd } from 'react-icons/io';

export const AddNewOption: React.FC<{ action: () => void, label: string }> = ({ action, label }) => {
    return (
        <div
            className='flex items-center gap-2 p-select font-semibold transition-all hover:bg-slate-200 cursor-pointer'
            onClick={action}
        >
            <IoMdAdd size="20px" />
            {label}
        </div>
    )
}
