import { AccountNavigation } from '@components/Account/Navigation';
import React from 'react'
import { Outlet } from 'react-router-dom';

const Account = () => {
    return (
        <section>
            <AccountNavigation />
            <Outlet />
        </section>
    )
}

export default Account;