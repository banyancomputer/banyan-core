import React from 'react';
import { useIntl } from 'react-intl';

import { useModal } from '@/app/contexts/modals';

import { PlusBold, Upload } from '@static/images/common';

const Home = () => {
    const { openModal } = useModal();
    const { messages } = useIntl();

    return (
        <section className="py-9 pt-14 px-4" id="buckets">
            <div className="mb-4 flex flex-col w-full justify-between gap-4">
                <h2 className="text-lg font-semibold">
                    {`${messages.allDrives}`}
                </h2>
                <div className="flex items-stretch gap-2">
                    <button
                        className="btn-highlighted gap-2 w-[138px] py-2 px-4 text-sm"
                        onClick={() => ({})}
                    >
                        <Upload />
                        {`${messages.upload}`}
                    </button>
                    <button
                        className="flex items-center gap-2 py-2 px-4 border-1 border-border-regular rounded-md text-text-900 font-semibold"
                        onClick={() => ({})}
                    >
                        <PlusBold width="20px" height="20px" />
                        {`${messages.newDrive}`}
                    </button>
                </div>
            </div>
        </section>
    );
};

export default Home;
