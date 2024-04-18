import React, { useEffect, useState } from 'react';
import { useAppDispatch, useAppSelector } from '@app/store';
import { getAcceptedDeals } from '@app/store/deals/actions';
import { convertFileSize } from '@app/utils/storage';
import { getDateLabel } from '@app/utils/time';
import { SortCell } from '@components/common/SortCell';
import { ActiveDealsActions } from './Actions';

export const AcceptedDeals = () => {
    const dispatch = useAppDispatch();
    const { acceptedDeals } = useAppSelector(state => state.deals);
    const [sortState, setSortState] = useState({ criteria: '', direction: 'DESC' });

    const sort = (criteria: string) => {
        setSortState(prev => ({ criteria, direction: prev.direction === 'ASC' ? 'DESC' : 'ASC' }));
    };

    const reloadDeals = async () => {
        await dispatch(getAcceptedDeals());
    };


    useEffect(() => {
        reloadDeals();
    }, []);

    return (
        <div className="rounded-lg border-1 border-[#0000001A]">
            <table className="table w-full ">
                <thead className="bg-[#FAF5F0]">
                    <tr>
                        <th className="p-3 font-medium text-12">
                            <SortCell
                                criteria=''
                                onChange={sort}
                                sortState={sortState}
                                text="ID"
                            />
                        </th>
                        <th className="p-3 font-medium text-12">
                            <SortCell
                                criteria=''
                                onChange={sort}
                                sortState={sortState}
                                text="Size"
                            />
                        </th>
                        <th className="p-3 font-medium text-12">
                            <SortCell
                                criteria=''
                                onChange={sort}
                                sortState={sortState}
                                text="State"
                            />
                        </th>
                        <th className="p-3 font-medium text-12">
                            <SortCell
                                criteria=''
                                onChange={sort}
                                sortState={sortState}
                                text="Accepted By"
                            />
                        </th>
                        <th className="p-3 font-medium text-12">
                            <SortCell
                                criteria=''
                                onChange={sort}
                                sortState={sortState}
                                text="Accepted At"
                            />
                        </th>
                        <th className="p-3 text-12 text-left font-medium">Action</th>
                    </tr>
                </thead>
                <tbody className="bg-secondaryBackground text-[#3B3B3B]">
                    {
                        acceptedDeals.map(deal =>
                            <tr className="border-b-1 border-[#DDD] transition-all hover:bg-[#FFF3E6]">
                                <td className="p-3 text-14">{deal.id}</td>
                                <td className="p-3 text-14">{convertFileSize(+deal.size)}</td>
                                <td className="p-3 text-14">{deal.state}</td>
                                <td className="p-3 text-14">{deal.accepted_by}</td>
                                <td className="p-3 text-14">{deal.accepted_at ? getDateLabel(new Date(deal.accepted_at)) : 'N/A'}</td>
                                <td className="p-3 text-14">
                                <ActiveDealsActions dealId={deal.id} onDealAccepted={reloadDeals} />
                                </td>
                            </tr>
                        )
                    }
                </tbody>
            </table>
        </div>
    )
}
