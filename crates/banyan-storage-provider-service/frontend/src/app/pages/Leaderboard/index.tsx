import { LeaderboardTable } from "@components/Leaderboard/Table";
import { Select, Selectoption } from "@components/common/Select";
import { useState } from "react";

const Leaderboard = () => {
    const [timeRange, setTimeRange] = useState('weekly');
    const [location, setLocation] = useState('worldwide');

    const timeOptions = [
        new Selectoption('Monthly', 'monthly'),
        new Selectoption('Weekly', 'weekly'),
        new Selectoption('Daily', 'Daily'),
    ];

    const locationOptions = [
        new Selectoption('Worldwide', 'worldwide'),
        new Selectoption('America', 'america'),
        new Selectoption('Europe', 'europe'),
        new Selectoption('Asia', 'asia'),
    ];

    return <section>
        <div className="font-[350] font-boogy tracking-tighter text-lightText">
            <h2 className="mb-6 text-64 font-medium">Leaderboard</h2>
            <div className="mb-6 flex items-center justify-between">
                <h4 className="text-40">Top 10 Storage Providers</h4>
                <div className="flex items-center gap-2 font-inter text-darkText font-normal">
                    <Select
                        onChange={() => { }}
                        options={timeOptions}
                        placeholder={timeRange}
                        selectedOption={timeRange}
                    />
                    <Select
                        onChange={() => { }}
                        options={locationOptions}
                        placeholder={location}
                        selectedOption={location}
                    />
                </div>
            </div>
            <LeaderboardTable />
        </div>

    </section>
};

export default Leaderboard;