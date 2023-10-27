import { AreaChart, Area, XAxis, YAxis, ResponsiveContainer, Tooltip, Legend, CartesianGrid } from 'recharts';

import { Info } from '@static/images';


export const BandwidthUsage = () => {
    const data = [
        { 'Usage Data': 0, month: '1h' },
        { 'Usage Data': 6, month: '2h' },
        { 'Usage Data': 10, month: '3h' },
        { 'Usage Data': 7, month: '4h' },
        { 'Usage Data': -5.6, month: '5h' },
        { 'Usage Data': 8.75, month: '6h' },
        { 'Usage Data': -6, month: '7h' },
        { 'Usage Data': 8.8, month: '8h' },
        { 'Usage Data': 7, month: '9h' },
        { 'Usage Data': 5.6, month: '10h' },
        { 'Usage Data': 8.75, month: '11h' },
    ];

    const gradientOffset = () => {
        const dataMax = Math.max(...data.map(i => i['Usage Data']));
        const dataMin = Math.min(...data.map(i => i['Usage Data']));

        return dataMax / (dataMax - dataMin);
    };

    return (
        <div className=' w-2/5 bg-storageBackground text-lightText rounded-xl'>
            <div className='px-6 py-5 border-b-2 border-highlightColor'>
                <div className='flex items-center justify-between'>
                    <h4 className='text-20 font-medium'>Bandwidth usage of our service</h4>
                    <span className='text-highlightColor'>
                        <Info />
                    </span>
                </div>
                <p className='text-12'>Secondary text</p>
            </div>
            <div className='py-6 pb-10 pr-9 h-60'>
                <ResponsiveContainer>
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
                        <CartesianGrid
                            strokeDasharray="3 3"
                            stroke='#648B60'
                        />
                        <defs>
                            <linearGradient id="splitColor" x1="0" y1="0" x2="0" y2="1">
                                <stop offset={gradientOffset()} stopColor="#D99C67" stopOpacity={1} />
                                <stop offset={gradientOffset()} stopColor="#352F43" stopOpacity={1} />
                            </linearGradient>
                        </defs>
                        <Area
                            type="monotone"
                            dataKey="Usage Data"
                            fillOpacity={1}
                            stroke='transparent'
                            fill="url(#splitColor)"
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
