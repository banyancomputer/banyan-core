import React from 'react';
import dynamic from 'next/dynamic';

import { NextPageWithLayout } from './page';

import BaseLayout from '@layouts/BaseLayout';

// NOTE: we need to dynamically import the TombBucket module in order to use its wasm
const TombBucket = dynamic(
  () => import('@components/Buckets/TombBucket'),
  { ssr: false }
);

const Buckets: NextPageWithLayout = () => {
  return (
    <div>
      <div className="flex flex-col gap-2 p-6">
        <h1> Tomb Wasm stuff </h1>
        <TombBucket />
      </div>
    </div>
  )
}

export default Buckets

Buckets.getLayout = (page) => {
  return <BaseLayout>{page}</BaseLayout>;
};
