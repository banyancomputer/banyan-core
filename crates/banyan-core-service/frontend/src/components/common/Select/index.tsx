import React, { ReactElement, useEffect, useRef, useState } from 'react';
import { FiChevronDown } from 'react-icons/fi';
import { MdDone } from 'react-icons/md';
import { popupClickHandler } from '@/utils';

export class Selectoption {
    constructor(
        public label: string,
        public value: any
    ) { }
};

export interface SelectProps {
    options: Selectoption[];
    selectedOption: any;
    onChange: (option: any) => void;
    placeholder: string;
    initialOption?: ReactElement;
};

export const Select: React.FC<SelectProps> = ({ initialOption, onChange, options, placeholder, selectedOption }) => {
    const selectRef = useRef<HTMLDivElement | null>(null);
    const [isOptionstVisible, setIsOptionsVisible] = useState(false);

    const toggleSelect = () => {
        setIsOptionsVisible(prev => !prev);
    };

    const handleSelect = (option: Selectoption) => {
        onChange(option.value);
        setIsOptionsVisible(false);
    };

    const stopPropagation = (event: React.MouseEvent<HTMLUListElement>) => {
        event.stopPropagation();
    };

    useEffect(() => {
        const listener = popupClickHandler(selectRef.current!, setIsOptionsVisible);
        document.addEventListener('click', listener);

        return () => document.removeEventListener('click', listener);
    }, [selectRef]);

    return (
        <div
            ref={selectRef}
            onClick={toggleSelect}
            className="relative p-2.5 flex justify-between items-center text-sm font-medium border-1 border-gray-200 rounded-lg shadow-sm cursor-pointer select-none"
        >
            {selectedOption ? options.find(option => option.value === selectedOption)?.label : placeholder}
            <FiChevronDown
                className={`${isOptionstVisible && 'rotate-180'}`}
                stroke="#667085"
                size="20px"
            />
            {isOptionstVisible &&
                <ul
                    onClick={stopPropagation}
                    className="absolute left-0 top-12 w-full max-h-48 overflow-y-auto bg-white border-1 border-gray-200 rounded-lg shadow-sm z-10"
                >
                    {initialOption ? initialOption : null}
                    {options.map(option =>
                        <li
                            className="flex justify-between items-center p-2.5 transition-all hover:bg-slate-200 cursor-pointer"
                            key={option.value}
                            onClick={() => handleSelect(option)}
                        >
                            {option.label}
                            {selectedOption === option.value && <MdDone stroke="#667085" size="20px" />}
                        </li>
                    )}
                </ul>
            }
        </div>
    );
};
