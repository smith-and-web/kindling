import { Building2, MapPin, Package, Target, User } from "lucide-svelte";
import type { FieldEntityType, ReferenceTypeId } from "./types";

export interface ReferenceTypeOption {
  id: ReferenceTypeId;
  label: string;
  icon: typeof User;
  accentClass: string;
  bgClass: string;
  isDefault: boolean;
}

export const REFERENCE_TYPE_OPTIONS: ReferenceTypeOption[] = [
  {
    id: "characters",
    label: "Characters",
    icon: User,
    accentClass: "text-press-tag-orange",
    bgClass: "bg-press-tag-orange/20",
    isDefault: true,
  },
  {
    id: "locations",
    label: "Locations",
    icon: MapPin,
    accentClass: "text-press-tag-yellow",
    bgClass: "bg-press-tag-yellow/20",
    isDefault: true,
  },
  {
    id: "items",
    label: "Items",
    icon: Package,
    accentClass: "text-press-tag-green",
    bgClass: "bg-press-tag-green/20",
    isDefault: false,
  },
  {
    id: "objectives",
    label: "Objectives",
    icon: Target,
    accentClass: "text-press-tag-blue",
    bgClass: "bg-press-tag-blue/20",
    isDefault: false,
  },
  {
    id: "organizations",
    label: "Organizations",
    icon: Building2,
    accentClass: "text-press-tag-purple",
    bgClass: "bg-press-tag-purple/20",
    isDefault: false,
  },
  {
    id: "timelines",
    label: "Timelines",
    icon: Target,
    accentClass: "text-press-tag-blue",
    bgClass: "bg-press-tag-blue/20",
    isDefault: false,
  },
  {
    id: "custom",
    label: "Notes",
    icon: Package,
    accentClass: "text-press-tag-green",
    bgClass: "bg-press-tag-green/20",
    isDefault: false,
  },
];

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

export const REFERENCE_FIELD_TYPES: Record<ReferenceTypeId, FieldEntityType> = {
  characters: "character",
  locations: "location",
  items: "item",
  objectives: "objective",
  organizations: "organization",
  timelines: "timeline",
  custom: "custom",
};
