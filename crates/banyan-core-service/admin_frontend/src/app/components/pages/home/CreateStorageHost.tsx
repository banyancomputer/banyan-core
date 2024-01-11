import React, { useMemo, useState } from 'react';
import { createMemoryRouter, useNavigate } from 'react-router-dom';

import { SubmitButton } from '@components/common/SubmitButton';

import { useModal } from '@app/contexts/modals';
import { ToastNotifications } from '@app/utils/toastNotifications';
import { AdminClient } from '@/api/admin';
import { StorageHostRequest } from '@app/types';

const validUrl = (url: string): boolean => {
	if (!url) return false;

	try {
		new URL(url);
		return true;
	} catch (e) {
		return false;
	}
};

export const CreateStorageHost = ({
	client,
	onCreate,
}: {
	client: AdminClient;
	onCreate: () => Promise<void>;
}) => {
	const { closeModal } = useModal();
	const navigate = useNavigate();
	const [name, setName] = useState('');
	const [url, setUrl] = useState('');
	const [availableStorage, setAvailableStorage] = useState(549755813888000);
	const isDataValid = useMemo(
		() => name.length >= 3 && validUrl(url) && availableStorage > 0,
		[name, url, availableStorage]
	);

	const changeName = (event: React.ChangeEvent<HTMLInputElement>) => {
		const regexp = new RegExp(/^.{0,32}$/);
		if (!regexp.test(event.target.value)) {
			return;
		}

		setName(event.target.value);
	};

	const changeUrl = (event: React.ChangeEvent<HTMLInputElement>) => {
		setUrl(event.target.value);
	};
	const changeAvailableSpace = (event: React.ChangeEvent<HTMLInputElement>) => {
		const availableSpace = Number(event.target.value);
		if (isNaN(availableSpace) || availableSpace < 0) {
			return;
		}

		setAvailableStorage(availableSpace);
	};

	const create = async () => {
		const storageHost: StorageHostRequest = {
			name: name,
			url,
			available_storage: availableStorage,
		};
		try {
			await client.createStorageHost(storageHost);
			await onCreate();
			closeModal();
			navigate(`/`);
		} catch (error: any) {
			ToastNotifications.error(
				`Could not create storage host ${error.message}`,
				'Try again',
				create
			);
		}
	};

	return (
		<div className="w-modal flex flex-col gap-5">
			<div>
				<h4 className="text-m font-semibold ">{'New Storage Host'}</h4>
			</div>
			<div>
				<label>
					{}
					<input
						className="mt-2 input w-full h-11 py-3 px-4 rounded-md border-1 border-border-darken focus:outline-none"
						type="text"
						placeholder={'Host name'}
						value={name}
						onChange={changeName}
					/>
				</label>
			</div>
			<div>
				<label>
					{}
					<input
						className="mt-2 input w-full h-11 py-3 px-4 rounded-md border-1 border-border-darken focus:outline-none"
						type="text"
						placeholder={'Url'}
						value={url}
						onChange={changeUrl}
					/>
				</label>
			</div>
			<div>
				<label>
					{}
					<input
						className="mt-2 input w-full h-11 py-3 px-4 rounded-md border-1 border-border-darken focus:outline-none"
						type="number"
						placeholder={'Available Space'}
						value={availableStorage}
						onChange={changeAvailableSpace}
					/>
				</label>
			</div>
			<div className="flex items-center gap-3 text-xs">
				<button
					className="btn-secondary flex-grow py-3 px-4"
					onClick={closeModal}
				>
					{'Cancel'}
				</button>
				<SubmitButton text={'Create'} action={create} disabled={!isDataValid} />
			</div>
		</div>
	);
};
