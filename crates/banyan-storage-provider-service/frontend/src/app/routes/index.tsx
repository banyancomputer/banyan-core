import { lazy } from 'react';
import { useRoutes } from 'react-router-dom';

/** Pages */
const Dashboard = lazy(() => import('@pages/Dashboard'));
const Leaderboard = lazy(() => import('@pages/Leaderboard'));

/**
 * Route describes location mapping with components.
 */
class Route {
    constructor(
        public path: string,
        public element: JSX.Element,
        public children?: Route[],
        public fullPath: string = '',
    ) {
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
    public static Dashboard = new Route('/', <Dashboard />);
    public static Leaderboard = new Route('/leaderboard', <Leaderboard />);

    /** Routes is an array of logical router components */
    public static routes: Route[] = [
        RoutesConfig.Dashboard,
        RoutesConfig.Leaderboard
    ];
}

export const Routes = () => {
    const routes = useRoutes(RoutesConfig.routes);

    return routes;
};
