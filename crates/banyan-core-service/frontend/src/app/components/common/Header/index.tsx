import React, { useEffect, useRef, useState } from 'react';
import { useIntl } from 'react-intl';
import { useNavigate } from 'react-router-dom';

import { SearchInput } from '../SearchInput';

import { popupClickHandler } from '@/app/utils';
import { useKeystore } from '@/app/contexts/keystore';
import { Action } from '../FileActions';
import { useSession } from '@app/contexts/session';
import { HttpClient } from '@/api/http/client';

import { Key, LogoutAlternative, Settings } from '@static/images/common';

// TODO: Is this header used anywhere?
export const Header = () => {
	const userControlsRef = useRef<HTMLDivElement | null>(null);
	const { purgeKeystore } = useKeystore();
	const { messages } = useIntl();
	/** TODO: rework session logic. */
	const { userData } = useSession();
	const navigate = useNavigate();
	const [areProfileOptionsVisible, setAreProfileOptionsVisible] = useState(false);

	const toggleProfileOptionsVisibility = () => {
		setAreProfileOptionsVisible(prev => !prev);
	};

	const logout = async () => {
		let api = new HttpClient;
		try {
			console.log("logging out");
			await api.get('/auth/logout');
			await purgeKeystore();
			goTo('/');
		}
		catch (err: any) {
			console.error("An Error occurred trying to logout: ", err.message);
		}
	};

	const goTo = (path: string) => function () {
		navigate(path);
	};

	const options = [
		new Action(`${messages.settings}`, <Settings />, goTo('/account/settings')),
		new Action(`${messages.manageKeys}`, <Key />, goTo('/account/manage-keys')),
		new Action(`${messages.logout}`, <LogoutAlternative />, logout),
	];

	useEffect(() => {
		const listener = popupClickHandler(userControlsRef.current!, setAreProfileOptionsVisible);
		document.addEventListener('click', listener);

		return () => {
			document.removeEventListener('click', listener);
		};
	}, [userControlsRef]);

	return (
		<header className="flex items-center justify-between p-4 bg-mainBackground">
			{/* <SearchInput /> */}
			<div className="flex flex-grow items-center justify-end gap-6">
				<div
					className="relative w-10 h-10 rounded-full cursor-pointer "
					onClick={toggleProfileOptionsVisibility}
					ref={userControlsRef}
				>
					{userData?.user?.profileImage ?
						< img
							className="rounded-full"
							src={userData?.user.profileImage}
							width={40}
							height={40}
							alt="User Avatar"
						/>
						:
						null
					}
					{areProfileOptionsVisible &&
						<div
							className="absolute z-10 right-0 top-12 flex flex-col items-stretch shadow-xl rounded-xl text-xs font-semibold overflow-hidden  bg-bucket-actionsBackground text-bucket-actionsText cursor-pointer border-1 border-border-darken"
						>
							{options.map(option =>
								<div
									key={option.label}
									className="flex items-center gap-2 py-2.5 px-3 whitespace-nowrap transition-all hover:bg-hover"
									onClick={option.value}
								>
									<span className="text-button-primary">
										{option.icon}
									</span>
									{option.label}
								</div>
							)}
						</div>
					}
				</div>
			</div>
		</header >
	);
};