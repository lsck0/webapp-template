import { StrictMode, useState } from "react";
import { createRoot } from "react-dom/client";
import "@/global.scss";
import "@/tailwind.css";
import AppRouter from "@/router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ThemeProvider } from "@/components/ui/theming";
import { Toaster } from "@/components/ui/sonner";
import TypesafeI18n from "@/i18n/i18n-react";
import { useEffect } from "react";
import { detectLocale } from "@/i18n/detect";
import { loadFormatters, loadLocale } from "@/i18n/i18n-util.sync";

const queryClient = new QueryClient();
const detectedLocale = detectLocale();

export function App() {
    const [wasLocaleLoaded, setWasLocaleLoaded] = useState<Readonly<boolean>>(false);

    useEffect(() => {
        loadFormatters(detectedLocale);
        loadLocale(detectedLocale);
        setWasLocaleLoaded(true);
    }, []);

    if (!wasLocaleLoaded) return null;

    return (
        <StrictMode>
            <QueryClientProvider client={queryClient}>
                <ThemeProvider defaultTheme="system" storageKey="ui-theme">
                    <TypesafeI18n locale={detectedLocale}>
                        <AppRouter />
                        <Toaster duration={2000} />
                    </TypesafeI18n>
                </ThemeProvider>
            </QueryClientProvider>
        </StrictMode>
    );
}

createRoot(document.getElementById("root")!).render(<App />);
