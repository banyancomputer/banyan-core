import { AreaChart, Area, XAxis, YAxis, ResponsiveContainer, Tooltip, Legend, CartesianGrid } from 'recharts';

import { Info } from '@static/images';


export const StorageUsage = () => {
    const data = [
        { 'Current storage use': 0, 'Capacity': 0, month: 'Jan' },
        { 'Current storage use': 6, 'Capacity': 2.5, month: 'Feb' },
        { 'Current storage use': 10, 'Capacity': 5, month: 'Mar' },
        { 'Current storage use': 7, 'Capacity': 3, month: 'Apr' },
        { 'Current storage use': 5.6, 'Capacity': 1.25, month: 'Jun' },
        { 'Current storage use': 8.75, 'Capacity': 4.6, month: 'Jul' },
        { 'Current storage use': 6, 'Capacity': 2.5, month: 'Feb' },
        { 'Current storage use': 8.8, 'Capacity': 5.1, month: 'Mar' },
        { 'Current storage use': 7, 'Capacity': 3, month: 'Apr' },
        { 'Current storage use': 5.6, 'Capacity': 1.25, month: 'Jun' },
        { 'Current storage use': 8.75, 'Capacity': 4.6, month: 'Jul' },
    ];

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
                        data={data}
                        style={{ padding: 'none' }}
                        margin={{ top: 0, left: -20, right: 0, bottom: 0 }}
                    >
                        <YAxis
                            tickCount={20}
                            tickLine={false}
                            fontSize={9}
                            axisLine={false}
                            interval={'preserveStartEnd'}
                        />
                        <XAxis
                            dataKey="month"
                            tickLine={false}
                            axisLine={false}
                            fontSize={9}
                        />
                        <Tooltip />
                        <CartesianGrid
                            strokeDasharray="3 3"
                            stroke='#648B60'
                        />
                        <Area
                            dataKey="Current storage use"
                            fill='#D99C67'
                            fillOpacity={1}
                            stroke="#D99C67"
                        />
                        <Area
                            dataKey="Capacity"
                            fill='#352F43'
                            fillOpacity={1}
                            stroke="#352F43"
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
