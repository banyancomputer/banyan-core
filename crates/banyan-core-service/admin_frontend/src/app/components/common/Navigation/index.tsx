import React, { useEffect, useRef, useState } from 'react';
import { Link, NavLink, useLocation } from 'react-router-dom';

import {
	ActiveDirectory,
	ChevronUp,
	Directory,
	Logo,
	DeleteHotData,
} from '@static/images/common';

export const Navigation = () => {
	const [isBucketsVisible, setIsBucketsVisible] = useState(false);
	const location = useLocation();
	const [droppedBucket, setDroppedBucket] = useState<null>(null);

	const toggleBucketsVisibility = (event: React.MouseEvent<HTMLDivElement>) => {
		event.stopPropagation();
		event.preventDefault();
		setIsBucketsVisible((prev) => !prev);
	};

	return (
		<nav className="flex flex-col w-navbar min-w-navbar bg-navigation-primary py-6 pt-12 px-4 text-navigation-text border-r-2 border-r-navigation-border text-xs">
			<Link to="/" className="mb-5 flex">
				<Logo width="155px" height="32px" />
			</Link>
			<div className="flex-grow py-6 border-t-2 border-navigation-separator text-navigation-text">
				<NavLink
					to={'/'}
					className={`flex items-center justify-between gap-3 py-2.5 px-3 w-full h-10  cursor-pointer rounded-md ${
						location.pathname === '/' && 'bg-navigation-secondary'
					}`}
				>
					<span className="text-text-900">
						<Directory />
					</span>
					<span className="flex-grow">Providers</span>
					<span
						onClick={toggleBucketsVisibility}
						className={`${!isBucketsVisible && 'rotate-180'} ${
							![].length && 'hidden'
						}`}
					>
						<ChevronUp />
					</span>
				</NavLink>
				<NavLink
					to={'/deals'}
					className={`flex items-center justify-between gap-3 py-2.5 px-3 w-full h-10  cursor-pointer rounded-md ${
						location.pathname === '/deals' && 'bg-navigation-secondary'
					}`}
				>
					<span className="text-text-900">
						<DeleteHotData />
					</span>
					<span className="flex-grow">Deals</span>
					<span
						onClick={toggleBucketsVisibility}
						className={`${!isBucketsVisible && 'rotate-180'} ${
							![].length && 'hidden'
						}`}
					>
						<ChevronUp />
					</span>
				</NavLink>

				{isBucketsVisible && <ul className="flex-col gap-2 px-2 text-xxs"></ul>}
			</div>
		</nav>
	);
};
