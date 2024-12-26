interface ImportMetaEnv {
    readonly MODE: string;

    readonly GIT_COMMIT: string;
}

interface ImportMeta {
    readonly env: ImportMetaEnv;
}
