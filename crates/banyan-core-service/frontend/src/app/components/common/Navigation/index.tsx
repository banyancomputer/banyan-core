import React, { useEffect, useState } from 'react';
import { Link, NavLink, useLocation } from 'react-router-dom';

import { LockedTooltip } from './LockedTooltip';

import { useTomb } from '@/app/contexts/tomb';
import { useFilesUpload } from '@app/contexts/filesUpload';
import { ToastNotifications } from '@utils/toastNotifications';
import { Bucket } from '@app/types/bucket';
import { preventDefaultDragAction } from '@utils/dragHandlers';
import { StorageUsage } from '../StorageUsage';
import { useAppSelector } from '@/app/store';

import { ActiveDirectory, ChevronUp, Directory, Logo } from '@static/images/common';

export const Navigation = () => {
	const { buckets } = useTomb();
	const { uploadFiles } = useFilesUpload();
	const [isBucketsVisible, setIsBucketsVisible] = useState(false);
	const messages = useAppSelector(state => state.locales.messages.coponents.common.navigation);
	const location = useLocation();

	const toggleBucketsVisibility = (event: React.MouseEvent<HTMLDivElement>) => {
		event.stopPropagation();
		event.preventDefault();
		setIsBucketsVisible(prev => !prev);
	};

	const handleDrop = async (event: React.DragEvent<HTMLAnchorElement>, bucket: Bucket) => {
		preventDefaultDragAction(event);

		if (!event?.dataTransfer.files.length) { return; }

		try {
			await uploadFiles(event.dataTransfer.files, bucket, []);
		} catch (error: any) {
			ToastNotifications.error(`${messages.uploadError}`, `${messages.tryAgain}`, () => { });
		};
	};

	const preventNavigation = (event: React.MouseEvent<HTMLAnchorElement, MouseEvent>, bucket: Bucket) => {
		!bucket.mount && event.preventDefault();
	};

	useEffect(() => {
		if (isBucketsVisible) { return; }

		buckets.length && setIsBucketsVisible(true);
	}, [buckets]);

	return (
		<nav className="flex flex-col w-navbar min-w-navbar bg-navigation-primary py-6 pt-8 px-4 text-navigation-text border-r-2 border-r-navigation-border text-xs">
			<Link
				to="/"
				className="mb-2 flex"
				aria-label="Banyan logo"
			>
				<Logo width="155px" />
			</Link>
			<div className="flex-grow py-6 text-navigation-text">
				<NavLink
					to={'/'}
					className={`flex items-center justify-between gap-3 py-2.5 px-3 w-full h-10  cursor-pointer rounded-md bg-navigation-primary ${location.pathname === '/' && 'bg-navigation-secondary'} transition-all hover:brightness-95 `}
				>
					<span className="text-text-900">
						{
							location.pathname === '/' ?
								<ActiveDirectory />
								:
								<Directory />
						}
					</span>
					<span className="flex-grow">
						{`${messages.allDrives}`}
					</span>
					<span
						onClick={toggleBucketsVisibility}
						className={`${!isBucketsVisible && 'rotate-180'} ${!buckets.length && 'hidden'}`}
					>
						<ChevronUp />
					</span>
				</NavLink>
				{
					isBucketsVisible &&
					<ul className="flex-col gap-2 max-h-[calc(100vh-360px)] h-full w-full overflow-y-auto overflow-x-visible px-2 text-xxs">
						{
							buckets.map(bucket =>
								<li key={bucket.id}>
									<NavLink
										id={bucket.id}
										to={bucket.locked ? '' : `/drive/${bucket.id}`}
										onDrag={preventDefaultDragAction}
										onDrop={event => handleDrop(event, bucket)}
										onClick={event => preventNavigation(event, bucket)}
										className={`flex items-center justify-between gap-2 w-full h-10 ${!bucket.mount && 'cursor-not-allowed'} bg-navigation-primary transition-all hover:brightness-95 ${bucket.locked ? 'cursor-not-allowed' : 'cursor-pointer'}`}
									>
										<span
											className={`w-full flex items-center gap-3 py-2 px-2 ${bucket.locked ? 'pr-8' : 'pr-2'} flex-grow whitespace-nowrap rounded-md ${location.pathname.includes(bucket.id) && 'bg-navigation-secondary'}`}
										>
											<span className={`${location.pathname.includes(bucket.id) ? 'text-text-900' : 'text-navigation-textSecondary'}`}>
												{
													location.pathname.includes(bucket.id) ?
														<ActiveDirectory />
														:
														<Directory />
												}
											</span>
											<div className="overflow-hidden text-ellipsis w-full">
												{bucket.name}
											</div>
											<div className='relative h-4 '>
												{bucket.locked && <LockedTooltip bucket={bucket} className="left-0 top-0" />}
											</div>
										</span>
									</NavLink>
								</li>
							)
						}
					</ul>
				}
			</div>
			<StorageUsage />
		</nav>
	);
};
