import { Outlet } from 'react-router-dom';

import { AccountNavigation } from '@components/Account/Navigation';

const Account = () =>
    <section>
        <AccountNavigation />
        <Outlet />
    </section>;


export default Account;
