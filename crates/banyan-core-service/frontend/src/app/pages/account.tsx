import { Outlet } from 'react-router-dom';

import { AccountNavigation } from '@components/Account/Navigation';

const Account = () =>
    <section className="flex-grow flex flex-col">
        <AccountNavigation />
        <Outlet />
    </section>;

export default Account;
