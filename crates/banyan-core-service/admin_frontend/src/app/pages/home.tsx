import React, { useEffect, useState } from 'react';

import { AdminClient } from '@/api/admin';
import { NotFoundError } from '@/api/http';
import { HttpClient } from '@/api/http/client';
import { StorageProvidersTable } from '@components/pages/home/StorageProvidersTable';
import { StorageHost } from '@app/types';
import { Upload } from '@static/images/common';
import { useModal } from '@app/contexts/modals';
import { CreateStorageHost } from '@components/pages/home/CreateStorageHost';
import { Simulate } from 'react-dom/test-utils';

const client = new AdminClient();

const Home = () => {
	const [providers, setProviders] = useState<StorageHost[]>([]);
	const { openModal } = useModal();
	useEffect(() => {
		loadStorageHosts();
	}, []);

	const loadStorageHosts = async () => {
		try {
			const providers = await client.getStorageHosts();
			setProviders(providers);
		} catch (error: any) {
			if (error instanceof NotFoundError) {
				const api = new HttpClient();
				await api.get('/auth/logout');
				window.location.href = '/login';
			}
		}
	};

	const createHost = () => {
		openModal(
			<CreateStorageHost client={client} onCreate={loadStorageHosts} />
		);
	};

	return (
		<section className="py-9 pt-14 px-4" id="providers">
			<StorageProvidersTable storageProviders={providers} />
			<div className="flex items-stretch gap-2 py-2 ">
				<button
					className="btn-highlighted gap-2 w-[250px] py-2 text-sm"
					onClick={createHost}
				>
					<Upload />
					{'Create Host'}
				</button>
			</div>
		</section>
	);
};

export default Home;
