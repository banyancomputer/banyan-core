import React from 'react';
import dynamic from 'next/dynamic';

import BaseLayout from '@layouts/BaseLayout';
import { NextPageWithLayout } from './page';


// NOTE: we need to dynamically import the TombBucket module in order to use its wasm
const TombBucket = dynamic(
    () => import('@components/Buckets/TombBucket'),
    { ssr: false }
);

const Buckets: NextPageWithLayout = () =>
    <div>
        <div className="flex flex-col gap-2 p-6">
            <h1> Tomb Wasm stuff </h1>
            <TombBucket />
        </div>
    </div>;


export default Buckets;

Buckets.getLayout = (page) => <BaseLayout>{page}</BaseLayout>;
