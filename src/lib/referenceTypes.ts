import { Building2, MapPin, Package, Target, User } from "lucide-svelte";
import type { ReferenceTypeId } from "./types";
import { t } from "./i18n";

export interface ReferenceTypeOption {
  id: ReferenceTypeId;
  label: string;
  /** One of this type, lowercase, for copy like "Add character". */
  singular: string;
  icon: typeof User;
  accentClass: string;
  bgClass: string;
  isDefault: boolean;
}

export const REFERENCE_TYPE_OPTIONS: ReferenceTypeOption[] = [
  {
    id: "characters",
    get label() {
      return t("logic.referenceType.characters");
    },
    singular: "character",
    icon: User,
    accentClass: "text-press-tag-orange",
    bgClass: "bg-press-tag-orange/20",
    isDefault: true,
  },
  {
    id: "locations",
    get label() {
      return t("logic.referenceType.locations");
    },
    singular: "location",
    icon: MapPin,
    accentClass: "text-press-tag-yellow",
    bgClass: "bg-press-tag-yellow/20",
    isDefault: true,
  },
  {
    id: "items",
    get label() {
      return t("logic.referenceType.items");
    },
    singular: "item",
    icon: Package,
    accentClass: "text-press-tag-green",
    bgClass: "bg-press-tag-green/20",
    isDefault: false,
  },
  {
    id: "objectives",
    get label() {
      return t("logic.referenceType.objectives");
    },
    singular: "objective",
    icon: Target,
    accentClass: "text-press-tag-blue",
    bgClass: "bg-press-tag-blue/20",
    isDefault: false,
  },
  {
    id: "organizations",
    get label() {
      return t("logic.referenceType.organizations");
    },
    singular: "organization",
    icon: Building2,
    accentClass: "text-press-tag-purple",
    bgClass: "bg-press-tag-purple/20",
    isDefault: false,
  },
  {
    id: "timelines",
    get label() {
      return t("logic.referenceType.timelines");
    },
    singular: "timeline",
    icon: Target,
    accentClass: "text-press-tag-blue",
    bgClass: "bg-press-tag-blue/20",
    isDefault: false,
  },
  {
    id: "custom",
    get label() {
      return t("logic.referenceType.custom");
    },
    singular: "note",
    icon: Package,
    accentClass: "text-press-tag-green",
    bgClass: "bg-press-tag-green/20",
    isDefault: false,
  },
];

const REFERENCE_TYPE_IDS = new Set<string>(REFERENCE_TYPE_OPTIONS.map((option) => option.id));

/**
 * Copy that names a reference type ("No characters yet", "Add character") has one catalog key
 * per type, `<base>.<type>`, so every language can make the words agree in gender, number and
 * case. A type the catalog doesn't know (or none) reads the neutral `<base>.any` wording.
 */
export function referenceTypeText(base: string, type: string | null | undefined): string {
  return t(`${base}.${type && REFERENCE_TYPE_IDS.has(type) ? type : "any"}`);
}

export const DEFAULT_REFERENCE_TYPES: ReferenceTypeId[] = REFERENCE_TYPE_OPTIONS.filter(
  (option) => option.isDefault
).map((option) => option.id);

export function normalizeReferenceTypes(types?: string[] | null): ReferenceTypeId[] {
  if (!types) return DEFAULT_REFERENCE_TYPES;
  const allowed = new Set(REFERENCE_TYPE_OPTIONS.map((option) => option.id));
  const seen = new Set<ReferenceTypeId>();
  const result: ReferenceTypeId[] = [];

  for (const type of types) {
    if (allowed.has(type as ReferenceTypeId)) {
      const cast = type as ReferenceTypeId;
      if (!seen.has(cast)) {
        seen.add(cast);
        result.push(cast);
      }
    }
  }

  return result;
}

export { REFERENCE_FIELD_TYPES } from "./referenceEntityTypes";
