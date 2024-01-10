import { useEffect, useRef, useState } from 'react';

import { ProfileControls } from './ProfileControls';

import { useSession } from '@app/contexts/session';
import { popupClickHandler } from '@/app/utils';

export const Header = () => {
	const profileOptionsRef = useRef<HTMLDivElement | null>(null);
	const helpOptionsRef = useRef<HTMLDivElement | null>(null);
	const { userData } = useSession();
	const [areProfileOptionsVisible, setAreProfileOptionsVisible] =
		useState(false);
	const [areHelpOptionsVisible, setAreHelpOptionsVisible] = useState(false);

	const toggleHelpOptionsVisibility = () => {
		setAreHelpOptionsVisible((prev) => !prev);
	};

	const toggleProfileOptionsVisibility = () => {
		setAreProfileOptionsVisible((prev) => !prev);
	};

	useEffect(() => {
		const profileOptionsListener = popupClickHandler(
			profileOptionsRef.current!,
			setAreProfileOptionsVisible
		);
		const helpOptionsListener = popupClickHandler(
			helpOptionsRef.current!,
			setAreHelpOptionsVisible
		);
		document.addEventListener('click', profileOptionsListener);
		document.addEventListener('click', helpOptionsListener);

		return () => {
			document.removeEventListener('click', profileOptionsListener);
			document.removeEventListener('click', helpOptionsListener);
		};
	}, [profileOptionsRef, helpOptionsRef]);

	return (
		<header className="flex items-center justify-between p-4 bg-mainBackground border-b-1 border-border-regular">
			{/* <SearchInput /> */}
			<div className="flex flex-grow items-center justify-end gap-6">
				<div
					className="relative w-10 h-10 rounded-full cursor-pointer "
					onClick={toggleProfileOptionsVisibility}
					ref={profileOptionsRef}
				>
					{userData?.user?.profileImage ? (
						<img
							className="rounded-full"
							src={userData?.user.profileImage}
							width={40}
							height={40}
							alt="User Avatar"
						/>
					) : null}
					{areProfileOptionsVisible && <ProfileControls />}
				</div>
			</div>
		</header>
	);
};
