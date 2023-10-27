import React, { useEffect, useRef, useState } from 'react';
import { Deal } from './Deal';

export interface ActiveDeal {
    negotiatedPrice: string;
    dealDuation: string;
    status: string;
    nfsPath: string;
    acceptedAt: string;
    expectedBy: string;
};

export const ActiveDeals = () => {
    const dealsRef = useRef<HTMLDivElement | null>(null);
    const [dealsScroll, setDealsScroll] = useState(0);

    const deals: ActiveDeal[] = new Array(8).fill({
        negotiatedPrice: '$24/TB',
        dealDuation: '1 Month',
        status: 'Waiting for Seal',
        nfsPath: 'nfs://10.100.50.120/deals/...',
        acceptedAt: '5:54 pm ET 05/10/2023',
        expectedBy: '5:54 pm ET 05/10/2023',
    });

    useEffect(() => {
        /** Weird typescript issue with scrollTop which exist, but not for typescript */
        // @ts-ignore
        dealsRef.current?.addEventListener('scroll', event => setDealsScroll(event.target.scrollTop));
    }, [dealsRef]);

    return (
        <section>
            <h2 className='mt-20 mb-10 text-42 text-darkText'>Active Deals</h2>
            <div className='max-h-table overflow-y-scroll pr-2' ref={dealsRef}>
                <div className='flex flex-col gap-2'>
                    {deals.map((deal, index) =>
                        <Deal {...deal} key={index} dealsRef={dealsRef} dealsScroll={dealsScroll} />
                    )}
                </div>
            </div>
        </section>
    );
};
