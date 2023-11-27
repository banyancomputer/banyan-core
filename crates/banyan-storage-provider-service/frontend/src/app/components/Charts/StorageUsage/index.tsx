import { useEffect, useMemo } from 'react';
import { AreaChart, Area, XAxis, YAxis, ResponsiveContainer, Tooltip, CartesianGrid } from 'recharts';
import { useAppDispatch, useAppSelector } from '@app/store';

import { getStorageUsage } from '@app/store/metrics/actions';
import { convertFileSize } from '@app/utils/storage';
import { getDateLabel } from '@app/utils/time';

import { Info } from '@static/images';

export const StorageUsage = () => {
    const dispatch = useAppDispatch();
    const { storageUsage } = useAppSelector(state => state.metrics);

    const tableData = useMemo(() => storageUsage.map(usage => ({ 'Current storage use': usage.used, 'Capacity': usage.available, date: getDateLabel(new Date(usage.date[0], 0, usage.date[1]), false) })), [storageUsage]);

    useEffect(() => {
        try {
            (async () => {
                await dispatch(getStorageUsage());
            })()
        } catch (error: any) { }
    }, []);

    return (
        <div className='w-3/5 bg-storageBackground text-lightText rounded-xl'>
            <div className='px-6 py-5 border-b-2 border-highlightColor'>
                <div className='flex items-center justify-between'>
                    <h4 className='text-20 font-medium'>Current storage use</h4>
                    <span className='text-highlightColor'>
                        <Info />
                    </span>
                </div>
                <p className='text-12'>Secondary text</p>
            </div>
            <div className='py-6 pb-10 pr-9 h-60' >
                <ResponsiveContainer >
                    <AreaChart
                        data={tableData}
                        style={{ padding: 'none' }}
                        margin={{ top: 5, left: -20, right: 0, bottom: 0 }}
                    >
                        <YAxis
                            tickCount={9}
                            tickLine={false}
                            fontSize={9}
                            axisLine={false}
                            interval={0}
                            tickFormatter={value => convertFileSize(value, 0)!}
                            width={80}
                        />
                        <XAxis
                            interval={1}
                            dataKey="date"
                            tickLine={false}
                            axisLine={false}
                            fontSize={9}
                        />
                        <Tooltip formatter={(value, name) => [convertFileSize(+value), name]} />
                        <CartesianGrid
                            strokeDasharray="3 3"
                            stroke='#648B60'
                        />
                        <Area
                            dataKey="Capacity"
                            fill='#352F43'
                            fillOpacity={1}
                            stroke="#352F43"
                        />
                        <Area
                            dataKey="Current storage use"
                            fill='#D99C67'
                            fillOpacity={1}
                            stroke="#D99C67"
                        />
                    </AreaChart>
                </ResponsiveContainer>
                <div className='flex items-center gap-10 pl-5 text-12'>
                    <div className='flex items-center gap-3'>
                        <span className='w-2 h-2 rounded-full  bg-chartLight'></span>
                        Current storage use
                    </div>
                    <div className='flex items-center gap-3'>
                        <span className='w-2 h-2 rounded-full  bg-chartDark'></span>
                        Capacity
                    </div>
                </div>
            </div>
        </div>
    )
}
