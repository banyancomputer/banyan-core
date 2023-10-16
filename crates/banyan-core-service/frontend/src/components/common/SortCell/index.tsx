import React from 'react';

import { ArrowDown } from '@static/images/common';

interface SortCellProps {
    text: string;
    criteria: string;
    onChange: (criteria: string) => void;
    sortState: { criteria: string; direction: string };
}

export const SortCell: React.FC<SortCellProps> = ({ criteria, onChange, sortState, text }) => {
    const isActive = criteria === sortState.criteria;

    return (
        <div
            className="flex items-center gap-1 cursor-pointer select-none"
            onClick={() => onChange(criteria)}
        >
            {text} <div className={isActive && sortState.direction === 'ASC'? 'rotate-180' : ''}>
                <ArrowDown />
            </div>
        </div>
    );
};
