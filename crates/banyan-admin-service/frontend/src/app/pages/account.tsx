import { Outlet } from 'react-router-dom';

const Account = () =>
    <section className="flex-grow flex flex-col">
        <Outlet />
    </section>;

export default Account;
