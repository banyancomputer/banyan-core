import { UploadFileModal } from '@components/common/Modal/UploadFileModal';
import { Fallback } from '@components/common/Fallback';
import { Bucket } from '@components/Home/Bucket';
import { CreateDriveModal } from '@components/common/Modal/CreateDriveModal';
import { EmptyState } from '@components/Home/EmptyState';

import { useAppDispatch, useAppSelector } from '../store';
import { openModal } from '@store/modals/slice';

import { PlusBold, Upload } from '@static/images/common';

const Home = () => {
    const dispatch = useAppDispatch();
    const { buckets, isLoading } = useAppSelector(state => state.tomb);
    const messages = useAppSelector(state => state.locales.messages.pages.home);

    const uploadFile = () => {
        dispatch(openModal({ content: <UploadFileModal path={[]} driveSelect /> }));
    };

    const createDrive = () => {
        dispatch(openModal({ content: <CreateDriveModal /> }));
    };

    return (
        <section className="h-[455px] py-9 pt-14 px-4" id="buckets">
            <div className="mb-4 flex flex-col w-full justify-between gap-4">
                <h2 className="text-lg font-semibold">
                    {`${messages.allDrives}`}
                </h2>
                {!isLoading ?
                    <div className="flex items-stretch gap-2">
                        <button
                            className="btn-primary gap-2 w-[138px] py-2 px-4 text-sm"
                            onClick={uploadFile}
                        >
                            <Upload />
                            {`${messages.upload}`}
                        </button>
                        <button
                            className="flex items-center gap-2 py-2 px-4 border-1 border-border-regular rounded-md text-text-900 font-semibold"
                            onClick={createDrive}
                        >
                            <PlusBold width="20px" height="20px" />
                            {`${messages.newDrive}`}
                        </button>
                    </div>
                    :
                    null
                }
            </div>
            <Fallback shouldRender={!isLoading}>
                {buckets.length ?
                    <div className="grid grid-cols-3 gap-3 pb-4 xl:grid-cols-4 ">
                        {
                            buckets.map(bucket =>
                                <Bucket bucket={bucket} key={bucket.id} />
                            )
                        }
                    </div>
                    :
                    <EmptyState />
                }
            </Fallback>
        </section>
    );
};

export default Home;
