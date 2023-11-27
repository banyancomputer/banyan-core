import React, { useEffect } from 'react';
import { Deal } from './Deal';
import { useAppDispatch, useAppSelector } from '@app/store';
import { getAvailableDeals } from '@app/store/deals/actions';

export const PotentialDeals = () => {
    const { availiableDeals } = useAppSelector(state => state.deals);
    const dispatch = useAppDispatch();

    useEffect(() => {
        try {
            (async () => {
                await dispatch(getAvailableDeals());
            })()
        } catch (error: any) { }
    }, []);

    return (
        <section>
            <h2 className='mt-20 mb-10 text-42 text-darkText'>Potential Deals</h2>
            <div className='max-h-table overflow-y-scroll pr-2'>
                <div className='flex flex-col gap-2'>
                    {availiableDeals.map((deal, index) =>
                        <Deal {...deal} key={index} />
                    )}
                </div>
            </div>
        </section>
    );
};
