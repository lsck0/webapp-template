import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useI18nContext } from "@/i18n/i18n-react";
import { Locales } from "@/i18n/i18n-types";
import { locales } from "@/i18n/i18n-util";
import { loadFormatters, loadLocale } from "@/i18n/i18n-util.sync";

export function LanguageToggle() {
    const { locale, setLocale } = useI18nContext();

    const changeLocale = (newLocale: Locales) => {
        if (newLocale === locale) return;

        setLocale(newLocale);
        localStorage.setItem("locale", newLocale);

        loadLocale(newLocale);
        loadFormatters(newLocale);
    };

    return (
        <Select onValueChange={changeLocale} value={locale}>
            <SelectTrigger className="w-18">
                <SelectValue placeholder={locale} />
            </SelectTrigger>
            <SelectContent>
                {locales.sort().map((locale: string) => (
                    <SelectItem key={locale} value={locale}>
                        {locale.toUpperCase()}
                    </SelectItem>
                ))}
            </SelectContent>
        </Select>
    );
}
