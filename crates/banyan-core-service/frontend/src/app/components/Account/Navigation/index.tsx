import { Link, useLocation } from 'react-router-dom';

import { useAppSelector } from '@/app/store';

class SettingsLink {
    constructor(
        public label: string,
        public path: string
    ) { }
};

export const AccountNavigation = () => {
    const { pathname } = useLocation();
    const messages = useAppSelector(state => state.locales.messages.coponents.account.navigation);

    const links = [
        // new SettingsLink(`${messages.profile}`, '/account/profile'),
        new SettingsLink(`${messages.profile}`, '/account/profile'),
        new SettingsLink(`${messages.manageAccessKeys}`, '/account/manage-keys'),
        new SettingsLink(`${messages.billingAndPayment}`, '/account/billing'),
        // new SettingsLink(`${messages.services}`, '/account/services'),
    ];

    return (
        <section className="py-5 px-10" id="buckets">
            <div className="mb-4 flex w-full justify-between items-center">
                <h2 className="text-xl font-semibold">
                    {`${messages.title}`}
                </h2>
            </div>
            <div className="border-b-1 border-border-regular">
                <ul className="w-max flex justify-between rounded-md">
                    {links.map(link =>
                        <li className="flex-grow" key={link.label}>
                            <Link
                                className={`relative flex justify-center w-full py-4 px-6 rounded-md text-xs transition-all ${pathname == link.path && 'font-semibold'} `}
                                to={link.path}
                            >
                                {link.label}
                                {pathname == link.path &&
                                    <span className="absolute bottom-0 left-0 w-full h-[2px] bg-text-900" />
                                }
                            </Link>
                        </li>
                    )}
                </ul>
            </div>
        </section>
    );
};

