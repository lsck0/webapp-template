import { createRoute } from "@tanstack/react-router";
import { rootRoute } from "@/router";

export const IndexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/",
    component: IndexRouteComponent,
});

export function IndexRouteComponent() {
    return (
        <>
            <title>Webapp Template</title>
            <p>Hello World</p>
            <p>Version: {import.meta.env.GIT_COMMIT}</p>
            <p>Last updated: {import.meta.env.LAST_UPDATED}</p>
        </>
    );
}
