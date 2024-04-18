import React, { useEffect, useState } from 'react';
import { useAppDispatch, useAppSelector } from '@app/store';
import { getAvailableDeals } from '@app/store/deals/actions';
import { convertFileSize } from '@app/utils/storage';
import { getDateLabel } from '@app/utils/time';
import { SortCell } from '@components/common/SortCell';
import { AvailiableDealsActions } from './Actions';

export const AvailableDeals = () => {
    const dispatch = useAppDispatch();
    const { availableDeals } = useAppSelector(state => state.deals);
    const [sortState, setSortState] = useState({ criteria: '', direction: 'DESC' });

    const sort = (criteria: string) => {
        setSortState(prev => ({ criteria, direction: prev.direction === 'ASC' ? 'DESC' : 'ASC' }));
    };

    useEffect(() => {
        (async () => {
            await dispatch(getAvailableDeals());
        })()
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
                                text="Negotiated Price"
                            />
                        </th>
                        <th className="p-3 font-medium text-12">
                            <SortCell
                                criteria=''
                                onChange={sort}
                                sortState={sortState}
                                text="Seal by"
                            />
                        </th>
                        <th className="p-3 font-medium text-12">
                            <SortCell
                                criteria=''
                                onChange={sort}
                                sortState={sortState}
                                text="Proposed FIL amount"
                            />
                        </th>
                        <th className="p-3 font-medium text-12">
                            <SortCell
                                criteria=''
                                onChange={sort}
                                sortState={sortState}
                                text="Duration"
                            />
                        </th>
                        <th className="p-3 font-medium text-12">
                            <SortCell
                                criteria=''
                                onChange={sort}
                                sortState={sortState}
                                text="Requested At"
                            />
                        </th>
                        <th className="p-3 text-12 text-left font-medium">Action</th>
                    </tr>
                </thead>
                <tbody className="bg-secondaryBackground text-[#3B3B3B]">
                    {
                        availableDeals.map(deal =>
                            <tr className="border-b-1 border-[#DDD] transition-all hover:bg-[#FFF3E6]">
                                <td className="p-3 text-14">{deal.id}</td>
                                <td className="p-3 text-14">{convertFileSize(+deal.size)}</td>
                                <td className="p-3 text-14">$24/TB</td>
                                <td className="p-3 text-14">{getDateLabel(new Date(deal.sealed_by))}</td>
                                <td className="p-3 text-14">10 FIL</td>
                                <td className="p-3 text-14">4 months</td>
                                <td className="p-3 text-14">{getDateLabel(new Date(deal.accept_by))}</td>
                                <td className="p-3 text-14">
                                    <AvailiableDealsActions />
                                </td>
                            </tr>
                        )
                    }
                </tbody>
            </table>
        </div>
    )
}
