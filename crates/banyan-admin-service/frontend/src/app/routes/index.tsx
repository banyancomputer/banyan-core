import { lazy } from 'react';
import { useRoutes } from 'react-router-dom';

const Home = lazy(() => import('@app/pages/home'));
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

    /** Routes is an array of logical router components */
    public static routes: Route[] = [
        RoutesConfig.Home,
    ];
}

export const Routes = () => {
    const routes = useRoutes(RoutesConfig.routes);

    return routes;
};
