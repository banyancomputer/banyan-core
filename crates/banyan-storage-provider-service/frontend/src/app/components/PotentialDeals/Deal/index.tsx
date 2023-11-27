import React, { useState } from 'react';

import { convertFileSize } from '@app/utils/storage';
import { AvailiableDeal } from '@/entities/deals';
import { useAppDispatch } from '@app/store';
import { acceptDeal, declineDeal, downloadDeal, getActiceDeals, getAvailableDeals } from '@app/store/deals/actions';
import { getDealDateLabel } from '@app/utils/time';
import { getUSDAmount } from '@app/utils/price';

import { ChevronDown, Download } from '@static/images';

export const Deal: React.FC<AvailiableDeal> =
    ({ id,
        size,
        accept_by,
        sealed_by,
        payment,
        status,
    }) => {
        const dispatch = useAppDispatch();
        const [isAdditionalWInfoVisible, setIsAdditionalInfoVisible] = useState(false);

        const toggleVisibility = () => {
            setIsAdditionalInfoVisible(prev => !prev);
        };

        const download = async () => {
            try {
                await dispatch(downloadDeal(id));
            } catch (erro: any) { }
        };

        const accept = async () => {
            try {
                await dispatch(acceptDeal(id));
                await getActiceDeals();
                await getAvailableDeals();
            } catch (erro: any) { }
        };

        const decline = async () => {
            try {
                await dispatch(declineDeal(id));
                await getAvailableDeals();
            } catch (erro: any) { }
        };


        return (
            <div className='rounded-xl overflow-hidden border-1 border-tableBorder transition-all' id={id}>
                <table className="w-full">
                    <thead className='bg-tableHead text-12'>
                        <tr>
                            <th className='p-3 text-justify border-1 border-t-0 border-l-0 border-tableBorder'>Size</th>
                            <th className='p-3 text-justify border-1 border-t-0 border-tableBorder'>Negotiated Price</th>
                            <th className='p-3 text-justify border-1 border-t-0 border-tableBorder'>Seal by</th>
                            <th className='p-3 text-justify border-1 border-t-0 border-r-0 border-tableBorder w-60'>Accept Deal?</th>
                        </tr>
                    </thead>
                    <tbody className='bg-tableBody text-14'>
                        <tr>
                            <td className='py-4 px-3'>{convertFileSize(+size)}</td>
                            <td className='py-4 px-3'>{`$${getUSDAmount(payment)}/TB`}</td>
                            <td className='py-4 px-3'>{getDealDateLabel(new Date(sealed_by))}</td>
                            <td className='py-4 px-3'>
                                <div className='flex items-center justify-between gap-2'>
                                    <button
                                        className='flex-grow px-5 py-2.5 border-1 border-highlightColor rounded-lg font-semibold'
                                        onClick={accept}
                                    >
                                        Accept
                                    </button>
                                    <button
                                        className='flex-grow px-5 py-2.5 border-1 border-redHighlightColor rounded-lg font-semibold'
                                        onClick={decline}
                                    >
                                        Decline
                                    </button>
                                </div>
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
                            <span className='w-32 text-12 text-tableExtendText'>Deal Duration</span>
                            <span className='text-14 font-medium'>{''}</span>
                        </div>
                        <div className='flex items-center gap-10 p-3'>
                            <span className='w-32 text-12 text-tableExtendText'>Deal Identifier</span>
                            <span className='text-14 font-medium'>{id}</span>
                        </div>
                    </div>
                }
            </div>
        )
    }
