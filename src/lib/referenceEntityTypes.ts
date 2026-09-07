import type { FieldEntityType, ReferenceTypeId } from "./types";

// Kept separate from the icon registry for backend-free development helpers.
export const REFERENCE_FIELD_TYPES: Record<ReferenceTypeId, FieldEntityType> = {
  characters: "character",
  locations: "location",
  items: "item",
  objectives: "objective",
  organizations: "organization",
  timelines: "timeline",
  custom: "custom",
};
