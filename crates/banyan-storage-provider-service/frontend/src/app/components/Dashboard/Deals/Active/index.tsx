import React, { useEffect, useState } from 'react';
import { useAppDispatch, useAppSelector } from '@app/store';
import { getActiveDeals } from '@app/store/deals/actions';
import { convertFileSize, convertToCurrency } from '@app/utils/storage';
import { getDateLabel } from '@app/utils/time';
import { SortCell } from '@components/common/SortCell';
import { ActiveDealsActions, REJECTED_DEALS_LOCAL_STORAGE_KEY } from './Actions';

export const ActiveDeals = () => {
    const dispatch = useAppDispatch();
    const { activeDeals } = useAppSelector(state => {
        let rejectedDeals = JSON.parse(localStorage.getItem(REJECTED_DEALS_LOCAL_STORAGE_KEY) || '[]');
        let { activeDeals } = state.deals;
        activeDeals = activeDeals.filter(deal => !rejectedDeals.includes(deal.id));
        return { activeDeals };

    });
    const [sortState, setSortState] = useState({ criteria: '', direction: 'DESC' });

    const sort = (criteria: string) => {
        setSortState(prev => ({ criteria, direction: prev.direction === 'ASC' ? 'DESC' : 'ASC' }));
    };

    const reloadDeals = async () => {
        await dispatch(getActiveDeals());
    };


    useEffect(() => {
        (async () => {
            await dispatch(getActiveDeals());
        })()
    }, []);

    return (
        <div className="rounded-lg border-1 border-[#0000001A]">
            <table className="table w-full ">
                <thead className="bg-[#FAF5F0]">
                <tr>
                    <th className="p-3 font-medium text-12">
                        <SortCell
                          criteria=""
                          onChange={sort}
                          sortState={sortState}
                          text="ID"
                        />
                    </th>
                    <th className="p-3 font-medium text-12">
                        <SortCell
                          criteria=""
                          onChange={sort}
                          sortState={sortState}
                          text="Size"
                        />
                    </th>
                    <th className="p-3 font-medium text-12">
                        <SortCell
                          criteria=""
                          onChange={sort}
                          sortState={sortState}
                          text="Negotiated Price"
                        />
                    </th>
                    <th className="p-3 font-medium text-12">
                        <SortCell
                          criteria=""
                          onChange={sort}
                          sortState={sortState}
                          text="Requested At"
                        />
                    </th>
                    <th className="p-3 font-medium text-12">
                        <SortCell
                          criteria=""
                          onChange={sort}
                          sortState={sortState}
                          text="Accept by"
                        />
                    </th>
                    <th className="p-3 text-12 text-left font-medium">Action</th>
                </tr>
                </thead>
                <tbody className="bg-secondaryBackground text-[#3B3B3B]">
                {
                    activeDeals.map(deal =>
                        <tr className="border-b-1 border-[#DDD] transition-all hover:bg-[#FFF3E6]">
                            <td className="p-3 text-14">{deal.id}</td>
                              <td className="p-3 text-14">{convertFileSize(+deal.size)}</td>
                              <td className="p-3 text-14">{convertToCurrency(deal.payment)}</td>
                              <td
                                className="p-3 text-14">{deal.requested_at ? getDateLabel(new Date(deal.requested_at)) : 'N/A'}</td>
                              <td
                                className="p-3 text-14">{deal.accept_by ? getDateLabel(new Date(deal.accept_by)) : 'N/A'}</td>
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