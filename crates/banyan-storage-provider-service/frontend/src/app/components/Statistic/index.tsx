import React, { useEffect } from 'react';

import { useAppDispatch, useAppSelector } from '@app/store'
import { getOveralStatistic } from '@app/store/metrics/actions';
import { convertFileSize } from '@app/utils/storage';

import { HealthStatus } from '@static/images';

export const Statistic = () => {
    const dispatch = useAppDispatch();
    const { overalStatistic } = useAppSelector(state => state.metrics);

    useEffect(() => {
        (async () => {
            await dispatch(getOveralStatistic());
        })()
    }, []);

    return (
        <section className='flex flex-col gap-3 text-18'>
            <h1 className='mb-20 text-80 font-light font-boogy tracking-tighter'>Storage Provider Management Dashboard</h1>
            <h3 className='mb-3 text-42 text-darkText'>Storage provider #134159</h3>
            <div className="flex items-center gap-6 text-darkText">
                <span className='font-light'>Currently used storage</span>
                <span className='font-normal'>{convertFileSize(overalStatistic.storage?.used)}</span>
            </div>
            <div className="flex items-center gap-6 text-darkText">
                <span className='font-light'>Avaiable/Allocated storage</span>
                <span className='font-normal'>{convertFileSize(overalStatistic.storage?.available)}</span>
            </div>
            <div className="flex items-center gap-6 text-darkText">
                <span className='font-light'>Start of billing period</span>
                <span className='font-normal'>04/04/2024</span>
            </div>
            <div className="flex items-start gap-6 text-darkText">
                <span className='font-light'>Ingress/ Egress Bandwidth <br />
                    <span className="text-14">since start of period</span>
                </span>
                <span className='font-normal'>{`Ingress ${convertFileSize(overalStatistic?.bandwidth?.ingress)}, Egress ${convertFileSize(overalStatistic?.bandwidth?.egress)}`}</span>
            </div>
            <div className="flex items-center gap-6 text-darkText">
                <span className='font-light'>Data sealed this period</span>
                <span className='font-normal'>{convertFileSize(overalStatistic?.deals?.sealed)}</span>
            </div>
            <p className='mb-12 mt-3 flex items-center gap-3'><HealthStatus />  Health Status</p>
        </section>
    )
}
