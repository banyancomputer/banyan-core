import { Deals } from '@components/Dashboard/Deals';
import { Charts } from '@components/Dashboard/Charts';
import { ServiceDetails } from '@components/Dashboard/ServiceDetails';
import { Statistic } from '@components/Dashboard/Statistic';

const DashBoard = () => {
    return <>
        <Statistic />
        <Charts />
        {/*<ServiceDetails />*/}
        <Deals />
    </>
};

export default DashBoard;