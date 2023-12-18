import { lazy } from 'react';
import { useRoutes } from 'react-router-dom';

const Home = lazy(() => import('@app/pages/home'));
const Bucket = lazy(() => import('@app/pages/bucket'));
const Account = lazy(() => import('@app/pages/account'));
const RegisterDevice = lazy(() => import('@app/pages/registerDevice'));
const Billing = lazy(() => import('@components/Account/Billing'));
const ManageKeys = lazy(() => import('@components/Account/ManageKeys'));
const Services = lazy(() => import('@components/Account/Services'));
const Settings = lazy(() => import('@components/Account/Settings'));

/**
 * Route describes location mapping with components.
 */
class Route {
    constructor(
        public path: string,
        public element: JSX.Element,
        public children?: Route[],
        public fullPath: string = ''
    ) { }

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
    public static Billing = new Route('billing', <Billing />);
    public static ManageKeys = new Route('manage-keys', <ManageKeys />);
    public static Services = new Route('services', <Services />);
    public static Settings = new Route('settings', <Settings />);

    /** Routes is an array of logical router components */
    public static routes: Route[] = [
        RoutesConfig.Home,
        RoutesConfig.Bucket,
        RoutesConfig.RegisterDevice,
        RoutesConfig.Account.addChildren([
            RoutesConfig.Billing,
            RoutesConfig.ManageKeys,
            RoutesConfig.Services,
            RoutesConfig.Settings,
        ]),
    ];
}

export const Routes = () => {
    const routes = useRoutes(RoutesConfig.routes);
    return routes;
};
