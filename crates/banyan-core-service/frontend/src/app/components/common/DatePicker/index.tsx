import React, { ReactElement, useEffect, useRef, useState } from 'react';
// @ts-ignore
import { DateRange } from 'react-date-range';

import { popupClickHandler } from '@/app/utils';

import { CalendarIcon, ChevronUp } from '@static/images/common';

export interface SelectProps {
    from: Date;
    to: Date;
    onChange: (startDate: Date, endDate: Date) => void;
    className?: string;
};

export const DatePicker: React.FC<SelectProps> = ({ onChange, from, to, className }) => {
    const datePickerRef = useRef<HTMLDivElement | null>(null);
    const [isOptionstVisible, setIsOptionsVisible] = useState(false);

    const toggleSelect = () => {
        setIsOptionsVisible(prev => !prev);
    };

    const stopPropagation = (event: React.MouseEvent<HTMLDivElement>) => {
        event.stopPropagation();
    };

    const changeDateRange = (dates: { startDate: Date; endDate: Date; key: string }) => {
        onChange(dates.startDate, dates.endDate);
    };

    useEffect(() => {
        const listener = popupClickHandler(datePickerRef.current!, setIsOptionsVisible);
        document.addEventListener('click', listener);

        return () => document.removeEventListener('click', listener);
    }, [datePickerRef]);

    return (
        <div
            ref={datePickerRef}
            onClick={toggleSelect}
            className={`relative p-2.5 flex justify-between items-center gap-2 text-sm cursor-pointer select-none text-text-900 ${className}`}
        >
            <CalendarIcon /> {`${from?.toLocaleDateString()} - ${to?.toLocaleDateString()}`}
            <span className={`${isOptionstVisible ? 'rotate-0' : 'rotate-180'}`}>
                <ChevronUp />
            </span>
            {isOptionstVisible &&
                <div
                    className="absolute right-0 top-14 border-1 border-border-darken rounded-lg shadow-sm overflow-hidden"
                    onClick={stopPropagation}
                >
                    <DateRange
                        ranges={[{
                            startDate: from,
                            endDate: to,
                            key: 'selection',
                        }]}
                        onChange={(item: any) => changeDateRange({ ...item.selection })}
                    />
                </div>
            }
        </div>
    );
};
