import { useEffect, useState } from 'react';
import { PrivateKey, TombFS, WasmBlockStore } from 'tomb-wasm-experimental';

// This is going to be initialized based on the bucket the user is currently in
const blockstoreUrl =
	'https://raw.githubusercontent.com/ipld/go-car/master/v2/testdata/sample-v2-indexless.car';
// This should NOT exist -- the privateKey should just be Crypto Key retrieved from keyStore!
const privateKeyUrl =
	'https://gist.githubusercontent.com/organizedgrime/f292f28a6ea39cea5fd1b844c51da4fb/raw/wrapping_key.pem';

export interface ITombBucket {}

const TombBucket: React.FC<ITombBucket> = () => {
    const [fs, setFs] = useState<TombFS | null>(null);
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
        <div>
            {fs ?
                <p> Filesystem properly Initialized </p>
			 :
                <p> Oops! Something went wrong with tomb-wasm </p>
            }
        </div>
    );
};
export default TombBucket;
