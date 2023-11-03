import { AccountNavigation } from '@components/Account/Navigation';
import React from 'react';
import { Outlet } from 'react-router-dom';

const Account = () =>
    <section>
        <AccountNavigation />
        <Outlet />
    </section>;


export default Account;
