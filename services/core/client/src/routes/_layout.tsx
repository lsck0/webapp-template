import { Outlet } from "@tanstack/react-router";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import Header from "@/components/header";

export default function GlobalLayout() {
    return (
        <>
            <Header />
            <Outlet />

            {import.meta.env.MODE === "development" && (
                <>
                    <TanStackRouterDevtools />
                    <ReactQueryDevtools initialIsOpen={false} />
                </>
            )}
        </>
    );
}
