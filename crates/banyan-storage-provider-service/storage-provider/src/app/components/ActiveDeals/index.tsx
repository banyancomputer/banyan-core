import React, { useEffect, useRef, useState } from 'react';
import { Deal } from './Deal';
import { useAppDispatch, useAppSelector } from '@app/store';
import { getActiceDeals } from '@app/store/deals/actions';
import { ActiveDeal } from '@/entities/deals';


export const ActiveDeals = () => {
    const dealsRef = useRef<HTMLDivElement | null>(null);
    const [dealsScroll, setDealsScroll] = useState(0);
    const dispatch = useAppDispatch();
    const { activeDeals } = useAppSelector(state => state.deals);

    useEffect(() => {
        /** Weird typescript issue with scrollTop which exist, but not for typescript */
        // @ts-ignore
        dealsRef.current?.addEventListener('scroll', event => setDealsScroll(event.target.scrollTop));
    }, [dealsRef]);

    useEffect(() => {
        try {
            (async () => {
                await dispatch(getActiceDeals());
            })()
        } catch (error: any) { }
    }, []);

    return (
        <section>
            <h2 className='mt-20 mb-10 text-42 text-darkText'>Active Deals</h2>
            <div className='max-h-table overflow-y-scroll pr-2' ref={dealsRef}>
                <div className='flex flex-col gap-2'>
                    {activeDeals.map((deal, index) =>
                        <Deal {...deal} key={index} dealsRef={dealsRef} dealsScroll={dealsScroll} />
                    )}
                </div>
            </div>
        </section>
    );
};
