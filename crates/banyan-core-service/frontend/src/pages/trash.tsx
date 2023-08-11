import React from 'react';

import { NextPageWithLayout } from './page';
import BaseLayout from '@/layouts/BaseLayout';
import { useTomb } from '@/contexts/tomb';

const Trash: NextPageWithLayout = () => {
    const { trash } = useTomb();

    return (
        <section className="py-9 px-4">
            <div className="mb-4 flex w-full justify-between items-center">
                <h2 className="text-xl font-semibold">
                    Trash
                </h2>
            </div>
        </section>
    );
};

export default Trash;

Trash.getLayout = (page) => <BaseLayout>{page}</BaseLayout>;
