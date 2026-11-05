/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Dev/QA override for the interface locale, e.g. `zh-CN`. Ignored in builds and tests. */
  readonly VITE_KINDLING_LOCALE?: string;
}
