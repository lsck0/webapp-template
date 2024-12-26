import { Locales } from "@/i18n/i18n-types";

export function detectLocale(): Locales {
    const availableLocales = ["en", "de"] as const;

    const storedLocale = localStorage.getItem("locale") as Readonly<string> | null;
    if (storedLocale && availableLocales.includes(storedLocale as Readonly<Locales>)) {
        return storedLocale as Readonly<Locales>;
    }

    const browserLocales = navigator.languages as Readonly<string[]> | null;
    for (const browserLocale of browserLocales || []) {
        const normalizedLocale = browserLocale.toLowerCase().split("-")[0] as Readonly<Locales>;

        if (availableLocales.includes(normalizedLocale)) return normalizedLocale;
    }

    return "en" as Readonly<Locales>;
}
