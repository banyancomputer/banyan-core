import React, { useEffect, useState } from 'react';
import { PrivateKey, TombFS, WasmBlockStore } from 'tomb-wasm-experimental';

import BaseLayout from '@layouts/BaseLayout';
import { NextPageWithLayout } from './page';

import { Add, Upload } from '@static/images/buckets';

// This is going to be initialized based on the bucket the user is currently in
const blockstoreUrl =
    'https://raw.githubusercontent.com/ipld/go-car/master/v2/testdata/sample-v2-indexless.car';
// This should NOT exist -- the privateKey should just be Crypto Key retrieved from keyStore!
const privateKeyUrl =
    'https://gist.githubusercontent.com/organizedgrime/f292f28a6ea39cea5fd1b844c51da4fb/raw/wrapping_key.pem';

const Buckets: NextPageWithLayout = () => {
    const [fs, setFs] = useState<TombFS | null>(null);

    const uploadFile = (event: React.ChangeEvent<HTMLInputElement>) => { };

    // Initialize the tombFs
    useEffect(() => {
        const initTombFs = async() => {
            let fs: TombFS | null = null;
            try {
                const bs = await WasmBlockStore.new(blockstoreUrl);
                const pkey = await PrivateKey.new(privateKeyUrl);
                fs = await TombFS.new(pkey, bs);
            } catch (err) {
                console.error(err);
            }
            setFs(fs);
        };
        initTombFs();
    }, []);

    return (
        <section className="py-9 px-4">
            <div className="mb-4 flex w-full justify-between items-center">
                <h2 className="text-xl font-semibold">
                    My Buckets
                </h2>
                <label className="flex gap-2 w-40 items-center justify-center py-2 px-4 font-semibold cursor-pointer rounded-lg bg-blue-primary text-white">
                    <Add />
                    Upload
                    <input
                        type="file"
                        className="hidden"
                        onChange={uploadFile}
                    />
                </label>
            </div>
            <label className="mt-10 flex flex-col items-center justify-center gap-4 px-6 py-4 border-2 border-c rounded-xl  text-xs cursor-pointer">
                <Upload />
                <span className="text-gray-600">
                    <span className="font-semibold text-black">Click to upload </span>
                    or drag and drop
                </span>
                <input
                    type="file"
                    className="hidden"
                    onChange={uploadFile}
                />
            </label>
        </section>
    );
};


export default Buckets;

Buckets.getLayout = (page) => <BaseLayout>{page}</BaseLayout>;
