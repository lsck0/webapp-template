import type { FormattersInitializer } from "typesafe-i18n";
import type { Locales, Formatters } from "./i18n-types.js";

export const initFormatters: FormattersInitializer<Locales, Formatters> = (locale: Locales) => {
    const formatters: Formatters = {
        number: (value: number, options?: Intl.NumberFormatOptions) => {
            return new Intl.NumberFormat(locale, options).format(value);
        },
        date: (date: Date) => {
            return new Intl.DateTimeFormat(locale).format(date);
        },
        currency: (value: number, currency: string, options?: Intl.NumberFormatOptions) => {
            return new Intl.NumberFormat(locale, { style: "currency", currency, ...options }).format(value);
        },
    };

    return formatters;
};
