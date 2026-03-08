interface ImportMetaEnv {
    readonly MODE: string;

    readonly GIT_COMMIT: string;
    readonly LAST_UPDATED: string;
}

interface ImportMeta {
    readonly env: ImportMetaEnv;
}
