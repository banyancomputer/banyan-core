import { BucketTable } from '@components/Bucket/Files/BucketTable';
import { Fallback } from '@components/common/Fallback';
import BucketHeader from '@components/Bucket/Files/Header';
import { EmptyState } from '@components/Bucket/Files/EmptyState';

import { useAppSelector } from '@store/index';

const BucketFiles = () => {
    const { selectedBucket, isLoading } = useAppSelector(state => state.tomb);

    return (
        <section className="py-9 px-10 flex flex-col flex-grow">
            <BucketHeader />
            <Fallback shouldRender={!isLoading}>
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