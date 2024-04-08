import { useEffect, useState } from 'react';
import { useAppDispatch, useAppSelector } from '@app/store';
import { getActiceDeals, getAvailableDeals } from '@app/store/deals/actions';
import { convertFileSize } from '@app/utils/storage';
import { getDateLabel } from '@app/utils/time';
import { ActiveDeals } from './Active';
import { AvailiableDeals } from './Availiable';


export const Deals = () => {
    const { activeDeals, availiableDeals } = useAppSelector(state => state.deals);
    const dispatch = useAppDispatch();
    const [dealsType, setDealsType] = useState<'potential' | 'active'>('potential');

    useEffect(() => {
        try {
            (async () => {
                dealsType === 'active' ?
                    await dispatch(getActiceDeals())
                    :
                    await dispatch(getAvailableDeals());
            })()
        } catch (error: any) { }
    }, [dealsType]);

    return (
        <section>
            <h2 className='mt-20 mb-10 text-28 text-darkText font-bold'>Deals</h2>
            <div className="mb-4 flex items-center justify-start gap-4 border-b-1 border-[#AAA] text-16 text-[#707070]">
                <div
                    className={`p-2 cursor-pointer ${dealsType === 'potential' ? 'text-[#274D5C] font-bold border-b-1 border-[#274D5C]' : ''}`}
                    onClick={() => setDealsType('potential')}
                >
                    Potential
                </div>
                <div
                    className={`p-2 cursor-pointer ${dealsType === 'active' ? 'text-[#274D5C] font-bold border-b-1 border-[#274D5C]' : ''}`}
                    onClick={() => setDealsType('active')}
                >
                    Active
                </div>
            </div>
            {dealsType === 'active' ?
                <ActiveDeals />
                :
                <AvailiableDeals />
            }
        </section>
    );
};
