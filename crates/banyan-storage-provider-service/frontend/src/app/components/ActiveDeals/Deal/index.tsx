import React, { useState } from 'react';
import { unwrapResult } from '@reduxjs/toolkit';

import { ActionsCell } from '@components/common/ActionsCell';

import { ActiveDeal } from '@/entities/deals';
import { getDealDateLabel } from '@app/utils/time';
import { useAppDispatch } from '@app/store';
import { downloadDeal, getActiceDeals, proofDeal } from '@app/store/deals/actions';
import { getUSDAmount } from '@app/utils/price';

import { ChevronDown, Download } from '@static/images';
import { Actions } from './Actions';

export const Deal: React.FC<ActiveDeal & { dealsRef: React.MutableRefObject<HTMLDivElement | null>, dealsScroll: number }> =
    ({
        id,
        size,
        payment,
        accepted_at,
        canceled_at,
        sealed_by,
        status,
        dealsRef,
        dealsScroll
    }) => {
        const [isAdditionalWInfoVisible, setIsAdditionalInfoVisible] = useState(false);
        const dispatch = useAppDispatch();

        const toggleVisibility = () => {
            setIsAdditionalInfoVisible(prev => !prev);
        };

        const download = async () => {
            try {
                const blob = unwrapResult(await dispatch(downloadDeal(id)));
                const link = document.createElement('a');
                const objectURL = URL.createObjectURL(blob);
                link.download = `${id}.car`;
                link.href = objectURL;
                document.body.appendChild(link);
                link.click();
            } catch (erro: any) { }
        };

        const proof = async () => {
            try {
                await dispatch(proofDeal(id));
                await getActiceDeals();
            } catch (erro: any) { }
        };

        return (
            <div className='rounded-xl border-1 border-tableBorder transition-all' id={id}>
                <table className="w-full">
                    <thead className='bg-tableHead text-12'>
                        <tr>
                            <th className='p-3 text-justify border-1 border-t-0 border-tableBorder'>Negotiated Price</th>
                            <th className='p-3 text-justify border-1 border-t-0 border-tableBorder'>Deal Duration</th>
                            <th className='p-3 text-justify border-1 border-t-0 border-r-0 border-tableBorder w-80'>Status</th>
                            <th className='p-3 text-justify border-1 border-t-0 border-r-0 border-tableBorder w-10'>Actions</th>
                        </tr>
                    </thead>
                    <tbody className='bg-tableBody text-14'>
                        <tr>
                            <td className='py-4 px-3'>{`$${getUSDAmount(payment)}/TB`}</td>
                            <td className='py-4 px-3'>{getDealDateLabel(new Date(accepted_at))}</td>
                            <td className='py-4 px-3 capitalize'>{status}</td>
                            <td className='py-4 px-3 cursor-pointer'>
                                <ActionsCell actions={<Actions />} />
                            </td>
                        </tr>
                    </tbody>
                    <tfoot>
                        <tr className='bg-tableBody text-14'>
                            <td
                                colSpan={5}
                                className='p-3 border-t-1 border-tableBorder cursor-pointer'
                                onClick={toggleVisibility}
                            >
                                <div
                                    className='flex items-center justify-between'
                                >
                                    <span>More</span>
                                    <span className={`${isAdditionalWInfoVisible && 'rotate-180'}`}><ChevronDown /></span>
                                </div>
                            </td>
                        </tr>
                    </tfoot>
                </table>
                {isAdditionalWInfoVisible &&
                    <div className='bg-tableExtend'>
                        <div className='flex items-center gap-10 p-3'>
                            <span className='w-32 text-12 text-tableExtendText'>Download Files</span>
                            <span className='flex items-center gap-3 text-14 font-medium cursor-pointer' onClick={download}>
                                <Download />
                                <span className='underline'>Download</span>
                            </span>
                        </div>
                        <div className='flex items-center gap-10 p-3'>
                            <span className='w-32 text-12 text-tableExtendText'>Proof</span>
                            <span className='text-14 font-medium underline' onClick={proof}>Click here for proof</span>
                        </div>
                        <div className='flex items-center gap-10 p-3'>
                            <span className='w-32 text-12 text-tableExtendText'>Accepted At</span>
                            <span className='text-14 font-medium'>{getDealDateLabel(new Date(accepted_at))}</span>
                        </div>
                        <div className='flex items-center gap-10 p-3'>
                            <span className='w-32 text-12 text-tableExtendText'>Expected Sealed By</span>
                            <span className='text-14 font-medium'>{getDealDateLabel(new Date(sealed_by))}</span>
                        </div>
                    </div>
                }
            </div>
        )
    }
