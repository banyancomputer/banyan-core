import { BucketTable } from '@components/Bucket/Files/BucketTable';
import { Fallback } from '@components/common/Fallback';
import BucketHeader from '@components/Bucket/Files/Header';
import { EmptyState } from '@components/Bucket/Files/EmptyState';

import { useTomb } from '@contexts/tomb';

const BucketFiles = () => {
    const { areBucketsLoading, selectedBucket } = useTomb();

    return (
        <section className="py-9 px-10 flex flex-col flex-grow">
            <BucketHeader />
            <Fallback shouldRender={!areBucketsLoading}>
                {selectedBucket &&
                    <>
                        {selectedBucket.files.length ?
                            <BucketTable bucket={selectedBucket} />
                            :
                            <EmptyState bucket={selectedBucket} />
                        }
                    </>
                }
            </Fallback>
        </section>
    )
}


export default BucketFiles;