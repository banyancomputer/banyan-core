import React, { ReactElement, useEffect, useRef, useState } from 'react';
import { popupClickHandler } from '@/app/utils';

import { ChevronUp, Done } from '@static/images/common';

export class Selectoption {
	constructor(
		public label: string,
		public value: any
	) {}
}

export interface SelectProps {
	options: Selectoption[];
	selectedOption: any;
	onChange: (option: any) => void;
	placeholder: string;
	initialOption?: ReactElement;
	className?: string;
}

export const Select: React.FC<SelectProps> = ({
	initialOption,
	onChange,
	options,
	placeholder,
	selectedOption,
	className,
}) => {
	const selectRef = useRef<HTMLDivElement | null>(null);
	const [isOptionstVisible, setIsOptionsVisible] = useState(false);

	const toggleSelect = () => {
		setIsOptionsVisible((prev) => !prev);
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
			className={`relative p-2.5 flex justify-between items-center text-sm font-medium border-1 border-border-darken rounded-md shadow-sm cursor-pointer select-none ${className}`}
		>
			{selectedOption
				? options.find((option) => option.value === selectedOption)?.label
				: placeholder}
			<ChevronUp className={`${!isOptionstVisible && 'rotate-180'}`} />
			{isOptionstVisible && (
				<ul
					onClick={stopPropagation}
					className="absolute left-0 top-12 w-full max-h-48 overflow-y-auto bg-secondaryBackground border-1 border-border-darken rounded-md shadow-sm z-10"
				>
					{initialOption ? initialOption : null}
					{options.map((option, index) => (
						<li
							className="flex justify-between items-center p-2.5 transition-all hover:bg-bucket-bucketHoverBackground cursor-pointer"
							key={index}
							onClick={() => handleSelect(option)}
						>
							{option.label}
							{selectedOption === option.value && (
								<Done width="20px" height="20px" />
							)}
						</li>
					))}
				</ul>
			)}
		</div>
	);
};
