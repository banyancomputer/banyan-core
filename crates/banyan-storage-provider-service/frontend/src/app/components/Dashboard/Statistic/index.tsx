import React, { useEffect } from 'react';

import { useAppDispatch, useAppSelector } from '@app/store'
import { getOverallStatistic } from '@app/store/metrics/actions';
import { convertFileSize } from '@app/utils/storage';

import { HealthStatus } from '@static/images';
import { Cell, Pie, PieChart, ResponsiveContainer } from 'recharts';

export const Statistic = () => {
    const dispatch = useAppDispatch();
    const { overalStatistic } = useAppSelector(state => state.metrics);

    const data = [
        { name: 'Free', value: overalStatistic.storage?.available - overalStatistic.storage?.used },
        { name: 'Used', value: overalStatistic.storage?.used },
    ];

    const COLORS = ['#E3E3E3', '#57221E'];

    useEffect(() => {
        (async () => {
            await dispatch(getOverallStatistic());
        })()
    }, []);

    return (
        <section className='mb-9 flex flex-col gap-3 text-16 text-darkText'>
            <h1 className='mb-20 text-64 font-light font-boogy tracking-tighter text-lightText'>Storage Provider Management Dashboard</h1>
            <div className="flex items-stretch gap-6">
                <div className="flex flex-col gap-6 w-1/2 p-6 rounded-lg bg-secondaryBackground">
                    <div className="relative h-48">
                        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 flex flex-col items-center gap-2">
                            <span className='text-14 font-bold'>{convertFileSize(overalStatistic.storage?.used)}</span>
                            <span className="text-12">{`of ${convertFileSize(overalStatistic.storage?.available)} used`}</span>
                        </div>
                        <ResponsiveContainer >
                            <PieChart >
                                <Pie
                                    data={data}
                                    innerRadius={70}
                                    outerRadius={90}
                                    dataKey="value"
                                    startAngle={90}
                                    endAngle={450}
                                >
                                    {data.map((_, index) => (
                                        <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                                    ))}
                                </Pie>
                            </PieChart>
                        </ResponsiveContainer>
                    </div>
                    <div className="flex items-center">
                        <div className="flex flex-col flex-grow">
                            <span>Ingress</span>
                            <span className="font-semibold">{convertFileSize(overalStatistic.bandwidth?.ingress)}</span>
                        </div>
                        <div className="flex flex-col flex-grow">
                            <span>Egress</span>
                            <span className="font-semibold">{convertFileSize(overalStatistic.bandwidth?.egress)}</span>
                        </div>
                        <div className="flex flex-col flex-grow">
                            <span>Next Billing Date</span>
                            <span className="font-semibold">-</span>
                        </div>
                        <div className="flex flex-col flex-grow">
                            <span>Data sealed this period</span>
                            <span className="font-semibold">{convertFileSize(overalStatistic.storage?.used)}</span>
                        </div>
                    </div>
                </div>
                <div className="flex flex-col gap-6 w-1/2 rounded-lg bg-secondaryBackground">
                    <div className="flex flex-col gap-6 flex-grow bg-secondaryBackground">
                        <div className="text-40 font-semibold">{overalStatistic.deals?.accepted}</div>
                        <div className="text-20">Active Deals</div>
                    </div>
                    <div className="flex flex-col gap-6 flex-grow bg-secondaryBackground">
                        <div className="text-40 font-semibold">{overalStatistic.deals?.sealed}</div>
                        <div className="text-20">Potential Deals</div>
                    </div>
                </div>
            </div>
        </section>
    )
}
