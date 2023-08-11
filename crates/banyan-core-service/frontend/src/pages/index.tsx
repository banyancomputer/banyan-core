import React from 'react';
import BaseLayout from '@layouts/BaseLayout';
import { NextPageWithLayout } from './page';
import TombBucket from '@/components/bucket/TombBucket';

const Buckets: NextPageWithLayout = () => (
	<div>
		<div className="flex flex-col gap-2 p-6">
			<h1> Tomb Wasm stuff </h1>
			<TombBucket bucket_id="test" />
			<TombBucket bucket_id="test2" />
		</div>
	</div>
);
export default Buckets;

Buckets.getLayout = (page: any) => <BaseLayout>{page}</BaseLayout>;
