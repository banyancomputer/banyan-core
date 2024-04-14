import { useEffect, useMemo } from 'react';
import { AreaChart, Area, XAxis, YAxis, ResponsiveContainer, CartesianGrid, Tooltip } from 'recharts';
import { useAppDispatch, useAppSelector } from '@app/store';

import { getBandwidthUsage } from '@app/store/metrics/actions';
import { convertFileSize } from '@app/utils/storage';

import { Info } from '@static/images';
import { getDateLabel } from '@app/utils/time';

export const BandwidthUsage = () => {
    const dispatch = useAppDispatch();
    const { bandWidthUsage } = useAppSelector(state => state.metrics);
    const tableData = useMemo(() => bandWidthUsage.map(usage => ({ 'Egress': usage.egress, 'Ingress': -usage.ingress, date: getDateLabel(new Date(usage.date[0], 0, usage.date[1]), false) })), [bandWidthUsage]);

    useEffect(() => {
        try {
            (async () => {
                await dispatch(getBandwidthUsage());
            })()
        } catch (error: any) { }
    }, []);

    return (
        <div className='w-1/2 bg-secondaryBackground text-lightText rounded-xl'>
            <div className='px-6 py-5 border-b-1 border-[#E1E1E1]'>
                <div className='flex items-center gap-4'>
                    <h4 className='text-20 font-medium'>Bandwidth usage of our service</h4>
                    <span className='text-highlightColor'>
                        <Info />
                    </span>
                </div>
                <p className='text-12'>Bandwidth since start of billing period</p>
            </div>
            <div className='py-6 pb-10 pr-9 h-60'>
                <ResponsiveContainer >
                    <AreaChart
                        data={tableData}
                        style={{ padding: 'none' }}
                        margin={{ top: 5, left: -20, right: 0, bottom: 0 }}
                    >
                        <YAxis
                            tickCount={9}
                            interval={0}
                            tickLine={false}
                            fontSize={9}
                            axisLine={false}
                            tickFormatter={value => convertFileSize(Math.abs(value), 0)!}
                            width={80}
                        />
                        <XAxis
                            dataKey="date"
                            tickLine={false}
                            axisLine={false}
                            fontSize={9}
                            interval={2}
                        />
                        <Tooltip formatter={(value, name) => [convertFileSize(Math.abs(+value)), name]} />
                        <CartesianGrid
                            strokeDasharray="3 3"
                            stroke='#374F35'
                        />
                        <Area
                            dataKey="Egress"
                            fill='#352F43'
                            fillOpacity={1}
                            stroke="#352F43"
                        />
                        <Area
                            dataKey="Ingress"
                            fill='#D99C67'
                            fillOpacity={1}
                            stroke="#D99C67"
                        />
                    </AreaChart>
                </ResponsiveContainer>
                <div className='flex items-center gap-10 pl-5 text-12'>
                    <div className='flex items-center gap-3'>
                        <span className='w-2 h-2 rounded-full  bg-chartLight'></span>
                        Outbound Data
                    </div>
                    <div className='flex items-center gap-3'>
                        <span className='w-2 h-2 rounded-full  bg-chartDark'></span>
                        Inbound Data
                    </div>
                </div>
            </div>
        </div >
    )
}
