import React from 'react';
import { useIntl } from 'react-intl';
import { FiAlertCircle, FiArrowRight } from "react-icons/fi"

import { NextPageWithLayout } from './page';
import BaseLayout from '@/layouts/BaseLayout';
import { useTomb } from '@/contexts/tomb';

const Trash: NextPageWithLayout = () => {
    const { trash } = useTomb();
    const { messages } = useIntl();

    return (
        <section className="py-9 px-4">
            <div className="mb-4">
                <h2 className="text-xl font-semibold">
                    {`${messages.trash}`}
                </h2>
            </div>
            <div className='flex items-start gap-3 p-4 border-1 rounded-lg bg-gray-200 border-gray-600'>
                <div><FiAlertCircle size="20px" /></div>
                <div className='text-xs'>
                    <h6 className='font-semibold'>Trash is full</h6>
                    <p className='mt-1 '>Click to empty trash.</p>
                    <button className='flex items-center gap-2 mt-3 font-semibold'>
                        Empty trash
                        <FiArrowRight size="20px" />
                    </button>
                </div>
            </div>
        </section>
    );
};

export default Trash;

Trash.getLayout = (page) => <BaseLayout>{page}</BaseLayout>;
