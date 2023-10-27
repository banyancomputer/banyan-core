import React, { useState } from 'react';

import { ActiveDeal } from '..';

import { ChevronDown, Dots, Download } from '@static/images';
import { ActionsCell } from '@components/common/ActionsCell';

export const Deal: React.FC<ActiveDeal & { dealsRef: React.MutableRefObject<HTMLDivElement | null>, dealsScroll: number }> =
    ({
        dealDuation,
        negotiatedPrice,
        acceptedAt,
        expectedBy,
        nfsPath,
        status,
        dealsRef,
        dealsScroll
    }) => {
        const [isAdditionalWInfoVisible, setIsAdditionalInfoVisible] = useState(false);

        const toggleVisibility = () => {
            setIsAdditionalInfoVisible(prev => !prev);
        };

        return (
            <div className='rounded-xl overflow-hidden border-1 border-tableBorder'>
                <table className="w-full">
                    <thead className='bg-tableHead text-12'>
                        <tr>
                            <th className='p-3 text-justify border-1 border-t-0 border-tableBorder'>Negotiated Price</th>
                            <th className='p-3 text-justify border-1 border-t-0 border-tableBorder'>Deal Duration</th>
                            <th className='p-3 text-justify border-1 border-t-0 border-r-0 border-tableBorder'>Status</th>
                            <th className='p-3 text-justify border-1 border-t-0 border-r-0 border-tableBorder w-10'>Actions</th>
                        </tr>
                    </thead>
                    <tbody className='bg-tableBody text-14'>
                        <tr>
                            <td className='py-4 px-3'>{negotiatedPrice}</td>
                            <td className='py-4 px-3'>{dealDuation}</td>
                            <td className='py-4 px-3'>{status}</td>
                            <td className='py-4 px-3 cursor-pointer'>
                                <ActionsCell
                                    actions={<></>}
                                    offsetTop={dealsScroll}
                                    tableRef={dealsRef}
                                />
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
                            <span className='flex items-center gap-3 text-14 font-medium cursor-pointer'>
                                <Download />
                                <span className='underline'>Download</span>
                            </span>
                        </div>
                        <div className='flex items-center gap-10 p-3'>
                            <span className='w-32 text-12 text-tableExtendText'>Proof</span>
                            <span className='text-14 font-medium underline'>Click here for proof</span>
                        </div>
                        <div className='flex items-center gap-10 p-3'>
                            <span className='w-32 text-12 text-tableExtendText'>NFS Path</span>
                            <span className='text-14 font-medium underline'>
                                <a href={nfsPath} target="_blank">
                                    {nfsPath}
                                </a>
                            </span>
                        </div>
                        <div className='flex items-center gap-10 p-3'>
                            <span className='w-32 text-12 text-tableExtendText'>Accepted At</span>
                            <span className='text-14 font-medium'>{acceptedAt}</span>
                        </div>
                        <div className='flex items-center gap-10 p-3'>
                            <span className='w-32 text-12 text-tableExtendText'>Expected Sealed By</span>
                            <span className='text-14 font-medium'>{expectedBy}</span>
                        </div>
                    </div>
                }
            </div>
        )
    }
