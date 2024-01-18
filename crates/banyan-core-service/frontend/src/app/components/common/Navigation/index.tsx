import React, { useEffect, useRef, useState } from 'react';
import { useIntl } from 'react-intl';
import { Link, NavLink, useLocation } from 'react-router-dom';

import { LockedTooltip } from './LockedTooltip';

import { useTomb } from '@/app/contexts/tomb';
import { useFilesUpload } from '@app/contexts/filesUpload';
import { ToastNotifications } from '@utils/toastNotifications';
import { Bucket } from '@app/types/bucket';
import { preventDefaultDragAction } from '@utils/dragHandlers';

import { ActiveDirectory, ChevronUp, Directory, Logo } from '@static/images/common';

export const Navigation = () => {
	const { buckets } = useTomb();
	const { uploadFiles, setFiles, files } = useFilesUpload();
	const [isBucketsVisible, setIsBucketsVisible] = useState(false);
	const { messages } = useIntl();
	const location = useLocation();
	const [droppedBucket, setDroppedBucket] = useState<null | Bucket>(null)

	const toggleBucketsVisibility = (event: React.MouseEvent<HTMLDivElement>) => {
		event.stopPropagation();
		event.preventDefault();
		setIsBucketsVisible(prev => !prev);
	};

	const handleDrop = async (event: React.DragEvent<HTMLAnchorElement>, bucket: Bucket) => {
		preventDefaultDragAction(event);

		if (!event?.dataTransfer.files.length) { return; }

		setFiles(Array.from(event.dataTransfer.files).map(file => ({ file, status: 'pending' })));
		setDroppedBucket(bucket!);
	};

	const preventNavigation = (event: React.MouseEvent<HTMLAnchorElement, MouseEvent>, bucket: Bucket) => {
		!bucket.mount && event.preventDefault();
	};

	useEffect(() => {
		if (!files.length || !droppedBucket) return;

		(async () => {
			try {
				ToastNotifications.uploadProgress();
				await uploadFiles(droppedBucket, []);
				setDroppedBucket(null);
			} catch (error: any) {
				setDroppedBucket(null);
				ToastNotifications.error(`${messages.uploadError}`, `${messages.tryAgain}`, () => { });
			}
		})()
	}, [files, droppedBucket]);

	useEffect(() => {
		if (isBucketsVisible) { return; }

		buckets.length && setIsBucketsVisible(true);
	}, [buckets]);

	return (
		<nav className="flex flex-col w-navbar min-w-navbar bg-navigation-primary py-6 pt-12 px-4 text-navigation-text border-r-2 border-r-navigation-border text-xs">
			<Link to="/" className="mb-5 flex" >
				<Logo width="155px" height="32px" />
			</Link>
			<div className="flex-grow py-6 border-t-2 border-navigation-separator text-navigation-text">
				<NavLink
					to={'/'}
					className={`flex items-center justify-between gap-3 py-2.5 px-3 w-full h-10  cursor-pointer rounded-md ${location.pathname === '/' && 'bg-navigation-secondary'}`}
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
					<ul className="flex-col gap-2 px-2 text-xxs">
						{
							buckets.map(bucket =>
								<li key={bucket.id}>
									<NavLink
										id={bucket.id}
										to={`/drive/${bucket.id}`}
										onDrag={preventDefaultDragAction}
										onDrop={event => handleDrop(event, bucket)}
										onClick={event => preventNavigation(event, bucket)}
										className={`relative flex items-center justify-between gap-2 w-full h-10 cursor-pointer ${!bucket.mount && 'cursor-not-allowed'}`}
									>
										<span
											className={`flex items-center gap-3 relative py-2 px-2 ${bucket.locked ? 'pr-6' : 'pr-2'} flex-grow whitespace-nowrap rounded-md overflow-ellipsis z-10 ${location.pathname.includes(bucket.id) && 'bg-navigation-secondary'}`}
										>
											<span className={`${location.pathname.includes(bucket.id) ? 'text-text-900' : 'text-navigation-textSecondary'}`}>
												{
													location.pathname.includes(bucket.id) ?
														<ActiveDirectory />
														:
														<Directory />
												}
											</span>
											{bucket.name}
											{bucket.locked && <LockedTooltip bucket={bucket} className="right-0" />}
										</span>
									</NavLink>
								</li>
							)
						}
					</ul>
				}
			</div>
		</nav>
	);
};
