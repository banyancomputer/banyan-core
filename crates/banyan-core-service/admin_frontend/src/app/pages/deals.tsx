import React, { useEffect, useState } from 'react';

import { AdminClient } from '@/api/admin';
import { NotFoundError } from '@/api/http';
import { HttpClient } from '@/api/http/client';
import { DealsTable } from '@components/pages/deals/DealsTable';
import { Deal } from '@app/types';
import { Upload } from '@static/images/common';
import { useModal } from '@app/contexts/modals';

const client = new AdminClient();

const Deals = () => {
	const [deals, setDeals] = useState<Deal[]>([]);
	const { openModal } = useModal();
	useEffect(() => {
		(async () => {
			try {
				const deals = await client.getDeals();
				setDeals(deals);
			} catch (error: any) {
				if (error instanceof NotFoundError) {
					const api = new HttpClient();
					await api.get('/auth/logout');
					window.location.href = '/login';
				}
			}
		})();
	}, []);

	const createDeal = () => {
		// openModal(<CreateDeal client={client}/>);
	};

	return (
		<section className="py-9 pt-14 px-4" id="deals">
			<DealsTable deals={deals} />
			<div className="flex items-stretch gap-2 py-2 ">
				<button
					className="btn-highlighted gap-2 w-[250px] py-2 text-sm"
					onClick={createDeal}
				>
					<Upload />
					{'Create Deals'}
				</button>
			</div>
		</section>
	);
};

export default Deals;
