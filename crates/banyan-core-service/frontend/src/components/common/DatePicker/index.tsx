import React, { ReactElement, useEffect, useRef, useState } from 'react';
import { FiChevronDown } from 'react-icons/fi';
import { popupClickHandler } from '@/utils';
//@ts-ignore
import { DateRange } from 'react-date-range';

export interface SelectProps {
    from: Date;
    to: Date
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

    const changeDateRange = (dates: { startDate: Date, endDate: Date, key: string }) => {
        onChange(dates.startDate, dates.endDate)
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
            className={`relative p-2.5 flex justify-between items-center text-sm font-medium border-1 border-inputBorder rounded-lg shadow-sm cursor-pointer select-none ${className}`}
        >
            {`${from?.toLocaleDateString()} - ${to?.toLocaleDateString()}`}
            <FiChevronDown
                className={`${isOptionstVisible && 'rotate-180'}`}
                stroke="#667085"
                size="20px"
            />
            {isOptionstVisible &&
                <div
                    className='absolute right-0 top-12 border-1 border-inputBorder rounded-lg shadow-sm overflow-hidden'
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
