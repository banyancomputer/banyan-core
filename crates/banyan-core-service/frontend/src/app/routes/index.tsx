import { useRoutes } from 'react-router-dom';

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
}

/**
 * RoutesConfig contains information about all routes and subroutes.
 */
export class RoutesConfig {
    /** Routes is an array of logical router components */
    public static routes: Route[] = [];
}

export const Routes = () => {
    const routes = useRoutes(RoutesConfig.routes);

    return routes;
};
