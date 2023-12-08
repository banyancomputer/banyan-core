import React, { useEffect, useRef, useState } from 'react';
import { useIntl } from 'react-intl';
import { Link, NavLink, useLocation } from 'react-router-dom';

import { LockedTooltip } from './LockedTooltip'
import { CreateBucketModal } from '../Modal/CreateBucketModal';

import { HttpClient } from '@/api/http/client';
import { useTomb } from '@/app/contexts/tomb';
import { useModal } from '@/app/contexts/modals';
import { useKeystore } from '@/app/contexts/keystore';
import { popupClickHandler } from '@/app/utils';
import { useFilesUpload } from '@app/contexts/filesUpload';
import { ToastNotifications } from '@app/utils/toastNotifications';
import { Bucket } from '@app/types/bucket';
import { preventDefaultDragAction } from '@app/utils/dragHandlers';

import { ChevronUp, Home, Info, Logo, Logout, Mail, Plus, Question, Trash } from '@static/images/common';

export const Navigation = () => {
	const { buckets } = useTomb();
	const { purgeKeystore } = useKeystore();
	const { uploadFiles, setFiles, files } = useFilesUpload();
	const [isBucketsVisible, setIsBucketsVisible] = useState(false);
	const [areHelpOpionsVisible, setAreHelpOpionsVisible] = useState(false);
	const { messages } = useIntl();
	const { openModal } = useModal();
	const helpRef = useRef<HTMLDivElement | null>(null);
	const location = useLocation();
	const [droppedBucket, setDroppedBucket] = useState<null | Bucket>(null)

	const toggleBucketsVisibility = (event: React.MouseEvent<HTMLDivElement>) => {
		event.stopPropagation();
		event.preventDefault();
		setIsBucketsVisible(prev => !prev);
	};

	const toggleHelpOptionsVisibility = (event: any) => {
		setAreHelpOpionsVisible(prev => !prev);
	};

	const createBucket = () => {
		openModal(<CreateBucketModal />);
	};

	const logout = async () => {
		let api = new HttpClient;
		try {
			await purgeKeystore();
			await api.get('/auth/logout');
			window.location.href = '/login';
		}
		catch (err: any) {
			console.error("An Error occurred trying to logout: ", err.message);
		}
	};

	const handleDrop = async (event: React.DragEvent<HTMLAnchorElement>, bucket: Bucket) => {
		preventDefaultDragAction(event);

		if (!event?.dataTransfer.files.length) { return; }

		setFiles(Array.from(event.dataTransfer.files).map(file => ({ file, isUploaded: false })));
		setDroppedBucket(bucket!);
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

	useEffect(() => {
		const listener = popupClickHandler(helpRef.current!, setAreHelpOpionsVisible);
		document.addEventListener('click', listener);

		return () => {
			document.removeEventListener('click', listener);
		};
	}, [helpRef]);

	return (
		<nav className="flex flex-col w-navbar min-w-navbar bg-navigation-primary py-8 px-4 text-navigation-text border-r-2 border-r-navigation-border">
			<Link to="/" className="mb-7 flex text-xs" >
				<Logo width="246px" height="56px" />
			</Link>
			<div className="flex-grow py-8 border-t-2 border-b-2 border-navigation-separator text-navigation-text">
				<NavLink
					to={'/'}
					className={'flex items-center justify-between gap-3 py-2.5 px-3 w-full h-10  cursor-pointer rounded-md bg-navigation-secondary'}
				>
					<Home />
					<span className="flex-grow">
						{`${messages.myDrives}`}
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
					<ul className="mt-3 mb-3 flex-col gap-2 px-4 text-xxs">
						{
							buckets.map(bucket =>
								<li key={bucket.id}>
									<NavLink
										id={bucket.id}
										to={`/drive/${bucket.id}`}
										onDrag={preventDefaultDragAction}
										onDrop={event => handleDrop(event, bucket)}
										className="relative flex items-center justify-between gap-2  w-full h-10  cursor-pointer"
									>
										<span className="absolute w-4 h-11 bottom-1/2 border-2 border-transparent border-l-navigation-secondary border-b-navigation-secondary">
										</span>
										<span
											className={` relative ml-5 py-2 px-2 ${bucket.locked ? 'pr-6' : 'pr-2'} flex-grow whitespace-nowrap rounded-md overflow-ellipsis z-10 ${location.pathname.includes(bucket.id) && 'bg-navigation-secondary'}`}
										>
											{bucket.name}
											{bucket.locked && <LockedTooltip bucket={bucket} />}
										</span>
									</NavLink>
								</li>
							)
						}
					</ul>
				}
				<button
					onClick={createBucket}
					className="mt-2 flex items-center gap-3 py-2 px-3 text-navigation-textSecondary"
				>
					<Plus />
					{`${messages.newDrive}`}
				</button>
			</div>
			<div className="flex flex-col gap-2 mt-6 pl-2 pt-3 pr-8 text-navigation-textSecondary text-xs">
				<span
					className="relative flex items-center gap-3 py-2.5 cursor-pointer"
					onClick={toggleHelpOptionsVisibility}
					ref={helpRef}
				>
					<Info />
					{`${messages.help}`}
					{areHelpOpionsVisible &&
						<div
							className="absolute left-0 top-10 w-full flex flex-col items-stretch shadow-xl rounded-xl overflow-hidden text-xs font-semibold overflow-hiddenaa bg-bucket-actionsBackground cursor-pointer text-bucket-actionsText"
						>
							<a
								className="flex items-center gap-2 py-2.5 px-3 transition-all hover:bg-hover"
								href="https://banyan8674.zendesk.com/hc/en-us"
								target="_blank"
							>
								<span className="text-button-primary">
									<Question />
								</span>
								FAQ
							</a>
							<a
								href="mailto:support@banyan8674.zendesk.com"
								className="flex items-center gap-2 py-2.5 px-3 transition-all hover:bg-hover"
								target="_blank"
							>
								<span className="text-button-primary">
									<Mail />
								</span>
								{`${messages.contactUs}`}
							</a>
						</div>
					}
				</span>
				<span
					className="flex items-center gap-3 py-2.5 cursor-pointer"
					onClick={logout}
				>
					<Logout />
					<span className="text-navigation-text">
						{`${messages.logoutAccount}`}
					</span>
				</span>
			</div>
		</nav>
	);
};

