import { useEffect, useState } from 'react';
import { useAppDispatch, useAppSelector } from '@app/store';
import { getAcceptedDeals, getAvailableDeals } from '@app/store/deals/actions';
import { convertFileSize } from '@app/utils/storage';
import { getDateLabel } from '@app/utils/time';
import { AcceptedDeals } from './Accepted';
import { AvailableDeals } from './Available';


export const Deals = () => {
    const { acceptedDeals, availableDeals } = useAppSelector(state => state.deals);
    const dispatch = useAppDispatch();
    const [dealsType, setDealsType] = useState<'active' | 'accepted'>('active');

    useEffect(() => {
        try {
            (async () => {
                dealsType === 'active' ?
                    await dispatch(getAcceptedDeals())
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
                    className={`p-2 cursor-pointer ${dealsType === 'active' ? 'text-[#274D5C] font-bold border-b-1 border-[#274D5C]' : ''}`}
                    onClick={() => setDealsType('active')}
                >
                    Potential
                </div>
                <div
                    className={`p-2 cursor-pointer ${dealsType === 'accepted' ? 'text-[#274D5C] font-bold border-b-1 border-[#274D5C]' : ''}`}
                    onClick={() => setDealsType('accepted')}
                >
                    Active
                </div>
            </div>
            {dealsType === 'accepted' ?
                <AcceptedDeals />
                :
                <AvailableDeals />
            }
        </section>
    );
};
