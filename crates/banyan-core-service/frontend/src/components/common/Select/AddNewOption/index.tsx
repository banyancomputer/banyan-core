import React from 'react';
import { IoMdAdd } from 'react-icons/io';

export const AddNewOption: React.FC<{ action: () => void; label: string }> = ({ action, label }) =>
    <div
        className="flex items-center gap-2 p-2.5 font-semibold transition-all hover:bg-hover cursor-pointer"
        onClick={action}
    >
        <IoMdAdd size="20px" />
        {label}
    </div>;

