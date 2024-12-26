import { ThemeToggle } from "@/components/ui/theming";
import { LanguageToggle } from "@/components/ui/language";

export default function Header() {
    return (
        <>
            <header className="bg-background shadow-secondary fixed z-10 flex h-12 w-full items-center justify-between p-4 shadow-md">
                <div className="flex items-center gap-10">
                    <p className="text-foreground text-md font-bold">Webapp Template</p>
                </div>

                <div className="flex items-center gap-4">
                    <LanguageToggle />
                    <ThemeToggle />
                </div>
            </header>

            <div className="h-12" />
        </>
    );
}
