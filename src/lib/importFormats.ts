import type { SourceType } from "./types";
import { t } from "./i18n";

interface ImportFormat {
  command: string;
  sourceType: SourceType;
  filters?: { name: string; extensions: string[] }[];
  directory?: boolean;
  label: string;
  references: boolean;
  sync: boolean;
}

const formats = {
  plottr: {
    command: "import_plottr",
    sourceType: "Plottr",
    filters: [{ name: "Plottr", extensions: ["pltr"] }],
    label: t("logic.importFormat.plottr"),
    references: true,
    sync: true,
  },
  markdown: {
    command: "import_markdown",
    sourceType: "Markdown",
    filters: [{ name: "Markdown", extensions: ["md", "markdown"] }],
    label: t("logic.importFormat.markdown"),
    references: false,
    sync: true,
  },
  ywriter: {
    command: "import_ywriter",
    sourceType: "YWriter",
    filters: [{ name: "yWriter 7", extensions: ["yw7"] }],
    label: t("logic.importFormat.ywriter"),
    references: true,
    sync: true,
  },
  longform: {
    command: "import_longform",
    sourceType: "Longform",
    filters: [{ name: "Longform Index", extensions: ["md", "markdown"] }],
    label: t("logic.importFormat.longform"),
    references: true,
    sync: true,
  },
  longformVault: {
    command: "import_longform",
    sourceType: "Longform",
    directory: true,
    label: t("logic.importFormat.longformVault"),
    references: true,
    sync: true,
  },
  novelwriter: {
    command: "import_novelwriter",
    sourceType: "NovelWriter",
    directory: true,
    label: t("logic.importFormat.novelwriter"),
    references: true,
    sync: true,
  },
  scrivener: {
    command: "import_scrivener",
    sourceType: "Scrivener",
    filters: [{ name: "Scrivener Project", extensions: ["scriv"] }],
    label: t("logic.importFormat.scrivener"),
    references: false,
    sync: false,
  },
} satisfies Record<string, ImportFormat>;

export type ImportType = keyof typeof formats;
export const IMPORT_FORMATS: Record<ImportType, ImportFormat> = formats;
export const IMPORT_COMMANDS = Object.fromEntries(
  Object.entries(formats).map(([key, config]) => [key, config.command])
) as Record<ImportType, string>;

export function isImportType(value: string): value is ImportType {
  return Object.prototype.hasOwnProperty.call(IMPORT_FORMATS, value);
}

export function importTypeForCommand(command: string): ImportType | undefined {
  return (Object.keys(IMPORT_FORMATS) as ImportType[]).find(
    (type) => IMPORT_FORMATS[type].command === command
  );
}

export function supportsSync(sourceType: SourceType): boolean {
  return Object.values(IMPORT_FORMATS).some(
    (format) => format.sourceType === sourceType && format.sync
  );
}
