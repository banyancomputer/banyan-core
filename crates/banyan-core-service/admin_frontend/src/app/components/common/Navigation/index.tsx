import React from 'react';
import { Link, NavLink, useLocation } from 'react-router-dom';

import { ChevronUp, DeleteHotData, Directory, Logo, Mail } from '@static/images/common';

export const Navigation = () => {
	const location = useLocation();


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
						<ChevronUp />
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
						<ChevronUp />
				</NavLink>

					<NavLink
						to={'/users'}
						className={`flex items-center justify-between gap-3 py-2.5 px-3 w-full h-10  cursor-pointer rounded-md ${
							location.pathname === '/users' && 'bg-navigation-secondary'
						}`}
					>
					<span className="text-text-900">
						<Mail />
					</span>
						<span className="flex-grow">Users</span>
						<ChevronUp />
				</NavLink>
			</div>
		</nav>
	);
};
