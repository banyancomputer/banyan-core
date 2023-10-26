import React from 'react';
import { Deal } from './Deal';

export interface PotentialDeal {
    size: number;
    negotiatedPrice: string;
    sealBy: string;
    proposedAmount: string;
    dealDuation: string;
    requestedAt: string,
    id: string
}

export const PotentialDeals = () => {
    const deals: PotentialDeal[] = new Array(8).fill({
        size: 1000000000,
        negotiatedPrice: '$24/TB',
        sealBy: '05.23.2023',
        proposedAmount: '10 FIL',
        dealDuation: '1 Month',
        requestedAt: '5:54 pm ET 05/10/2023',
        id: '109295759009823'
    });

    return (
        <section>
            <h2 className='mt-20 mb-10 text-42 text-darkText'>Potential Deals</h2>
            <div className='max-h-table overflow-y-scroll pr-2'>
                <div className='flex flex-col gap-2'>
                    {deals.map((deal, index) =>
                        <Deal {...deal} key={index} />
                    )}
                </div>
            </div>
        </section>
    );
};
