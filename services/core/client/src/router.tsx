import GlobalLayout from "@/routes/_layout";
import { IndexRoute } from "@/routes/index";
import { QueryClient, useQueryClient } from "@tanstack/react-query";
import { RouterProvider, createRouter } from "@tanstack/react-router";
import { createRootRouteWithContext } from "@tanstack/react-router";

// eslint-disable-next-line react-refresh/only-export-components
export const rootRoute = createRootRouteWithContext<{
    queryClient: QueryClient;
}>()({
    component: GlobalLayout,
    notFoundComponent: () => {
        const path = window.location.pathname;

        return <p>Error: Page "{path}" not found.</p>;
    },
    errorComponent: () => {
        return <p>Error: Something went wrong.</p>;
    },
});

const routeTree = rootRoute.addChildren([IndexRoute]);

const router = (queryClient: QueryClient) =>
    createRouter({
        routeTree,
        defaultPreload: "intent",
        defaultPreloadStaleTime: 0,
        context: {
            queryClient,
        },
        notFoundMode: "root",
    });

declare module "@tanstack/react-router" {
    interface Register {
        router: typeof router;
    }
}

export default function AppRouter() {
    const queryClient = useQueryClient();

    return <RouterProvider router={router(queryClient)} />;
}
