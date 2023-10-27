import { AreaChart, Area, XAxis, YAxis, ResponsiveContainer, Tooltip, Legend, CartesianGrid } from 'recharts';

import { Info } from '@static/images';

export const CurrentlyUsedStorage = () => {
    const data = [
        { 'Current storage use': 0, month: 'Jan' },
        { 'Current storage use': 6, month: 'Feb' },
        { 'Current storage use': 10, month: 'Mar' },
        { 'Current storage use': 7, month: 'Apr' },
        { 'Current storage use': 5.6, month: 'Jun' },
        { 'Current storage use': 8.75, month: 'Jul' },
        { 'Current storage use': 6, month: 'Feb' },
        { 'Current storage use': 8.8, month: 'Mar' },
        { 'Current storage use': 7, month: 'Apr' },
        { 'Current storage use': 5.6, month: 'Jun' },
        { 'Current storage use': 8.75, month: 'Jul' },
    ];

    return (
        <div className='mt-12 bg-storageBackground text-lightText rounded-xl'>
            <div className='px-6 py-5 border-b-2 border-highlightColor'>
                <div className='flex items-center justify-between'>
                    <h4 className='text-20 font-medium'>Currently used storage</h4>
                    <span className='text-highlightColor'>
                        <Info />
                    </span>
                </div>
                <p className='text-12'>TiB single value</p>
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
                            fill='#352F43'
                            fillOpacity={1}
                            stroke="#352F43"
                        />
                    </AreaChart>
                </ResponsiveContainer>
                <div className='flex items-center gap-10 pl-5 text-12'>
                    <div className='flex items-center gap-3'>
                        <span className='w-2 h-2 rounded-full bg-chartDark'></span>
                        Current storage use
                    </div>
                </div>
            </div>
        </div>
    )
}
