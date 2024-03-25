import { lazy } from 'react';
import { useRoutes } from 'react-router-dom';

import { CommonLayout } from "@app/layouts/common"

/** Pages */
const Home = lazy(() => import('@pages/home'));
const Bucket = lazy(() => import('@pages/bucket'));
const Account = lazy(() => import('@pages/account'));
const RegisterDevice = lazy(() => import('@pages/registerDevice'));

/** Components */
const Billing = lazy(() => import('@components/Account/Billing'));
const ManageKeys = lazy(() => import('@components/Account/ManageKeys'));
const Profile = lazy(() => import('@components/Account/Profile'));
const Snapshots = lazy(() => import('@components/Bucket/Snapshots'));
const Files = lazy(() => import('@components/Bucket/Files'));

/**
 * Route describes location mapping with components.
 */
class Route {
    element: JSX.Element;
    constructor(
        public path: string,
        component: JSX.Element,
        public Layout: React.FC<{ children: React.ReactNode }> | null = CommonLayout,
        public children?: Route[],
        public fullPath: string = '',
    ) {
        this.element = Layout ? <Layout>{component}</Layout> : component;
    }

    /** Adds routes array to children field, changed each route to add fullPath property */
    public addChildren(children: Route[]): Route {
        children.forEach(child => { child.fullPath = `${this.path}/${child.path}`; });
        this.children = children;

        return this;
    }
};

/**
 * RoutesConfig contains information about all routes and subroutes.
 */
export class RoutesConfig {
    public static Home = new Route('/', <Home />);
    public static Bucket = new Route('/drive/:id', <Bucket />);
    public static Account = new Route('/account', <Account />);
    public static RegisterDevice = new Route('/register-device/:spki', <RegisterDevice />);

    public static Billing = new Route('billing', <Billing />, null);
    public static ManageKeys = new Route('manage-keys', <ManageKeys />, null);
    public static Profile = new Route('profile', <Profile />, null);
    public static Snapshots = new Route('snapshots', <Snapshots />, null);
    public static Files = new Route('', <Files />, null);

    /** Routes is an array of logical router components */
    public static routes: Route[] = [
        RoutesConfig.Home,
        RoutesConfig.Bucket.addChildren([
            RoutesConfig.Files,
            RoutesConfig.Snapshots
        ]),
        RoutesConfig.RegisterDevice,
        RoutesConfig.Account.addChildren([
            RoutesConfig.Billing,
            RoutesConfig.ManageKeys,
            RoutesConfig.Profile,
        ]),
    ];
}

export const Routes = () => {
    const routes = useRoutes(RoutesConfig.routes);

    return routes;
};
