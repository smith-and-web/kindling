/** Browser development model. Native SQLite tests remain authoritative for transactions. */
import { REFERENCE_FIELD_TYPES } from "../lib/referenceEntityTypes";
import type {
  Project,
  ReferenceItem,
  ReferenceCopyRequest,
  ReferenceCopyPreview,
  FieldDefinition,
  FieldValue,
  Tag,
  EntityTag,
} from "../lib/types";
export interface CopyMockState {
  projects: Project[];
  references: ReferenceItem[];
  fields: FieldDefinition[];
  values: FieldValue[];
  tags: Tag[];
  assignments: EntityTag[];
}
const normalize = (s: string) => s.trim().toLowerCase();
function available(name: string, suffix: string, used: string[]) {
  let n = 1;
  while (true) {
    const value = `${name} (${suffix}${n > 1 ? ` ${n}` : ""})`;
    if (!used.some((s) => normalize(s) === normalize(value))) return value;
    n++;
  }
}
export function planMockCopy(state: CopyMockState, request: ReferenceCopyRequest) {
  const source = state.projects.find((p) => p.id === request.source_project_id);
  const destination = state.projects.find((p) => p.id === request.destination_project_id);
  if (!source || !destination || source.id === destination.id)
    throw new Error("Choose a different source project");
  const sourceRefs = state.references
    .filter((r) => r.project_id === source.id)
    .sort((a, b) =>
      `${a.reference_type}/${normalize(a.name)}/${a.id}`.localeCompare(
        `${b.reference_type}/${normalize(b.name)}/${b.id}`
      )
    );
  const selection =
    request.selection ?? sourceRefs.map(({ id, reference_type }) => ({ id, reference_type }));
  if (
    selection.some(
      (s) => !sourceRefs.some((r) => r.id === s.id && r.reference_type === s.reference_type)
    )
  )
    throw new Error("Selection contains missing references");
  const destinationNames = state.references
    .filter((r) => r.project_id === destination.id)
    .map((r) => ({ type: r.reference_type, name: r.name }));
  const names = [...destinationNames];
  const references: ReferenceCopyPreview["references"] = sourceRefs.map((r) => {
    const selected = selection.some((s) => s.id === r.id);
    const used = names.filter((n) => n.type === r.reference_type).map((n) => n.name);
    const conflict = destinationNames.some(
      (n) => n.type === r.reference_type && normalize(n.name) === normalize(r.name)
    );
    const copying = selected && (!conflict || request.keep_both.some((k) => k.id === r.id));
    const name =
      copying && used.some((n) => normalize(n) === normalize(r.name))
        ? available(r.name, "copy", used)
        : r.name;
    if (copying) names.push({ type: r.reference_type, name });
    return {
      id: r.id,
      reference_type: r.reference_type,
      name: r.name,
      description: r.description,
      selected,
      conflict,
      action: copying ? "copy" : selected ? "skip" : "unselected",
      destination_name: name,
    };
  });
  const rows = references.filter((r) => r.action === "copy");
  const ids = new Set(rows.map((r) => r.id));
  const types = new Set(rows.map((r) => REFERENCE_FIELD_TYPES[r.reference_type]));
  const fieldMap = new Map<string, string>();
  const fields: FieldDefinition[] = [];
  const changes: ReferenceCopyPreview["changes"] = [];
  for (const f of state.fields.filter(
    (f) => f.project_id === source.id && types.has(f.entity_type)
  )) {
    const candidates = [...state.fields.filter((d) => d.project_id === destination.id), ...fields];
    const matches = candidates.filter(
      (d) =>
        d.entity_type === f.entity_type &&
        d.name === f.name &&
        d.field_type === f.field_type &&
        JSON.stringify(d.options ? JSON.parse(d.options) : null) ===
          JSON.stringify(f.options ? JSON.parse(f.options) : null) &&
        d.default_value === f.default_value &&
        d.visible === f.visible &&
        d.required === f.required
    );
    if (matches.length === 1 && ![...fieldMap.values()].includes(matches[0].id)) {
      fieldMap.set(f.id, matches[0].id);
      continue;
    }
    const used = candidates.filter((d) => d.entity_type === f.entity_type).map((d) => d.name);
    const name = used.some((n) => normalize(n) === normalize(f.name))
      ? available(f.name, `from ${source.name}`, used)
      : f.name;
    const newField = {
      ...f,
      id: crypto.randomUUID(),
      project_id: destination.id,
      name,
      position:
        Math.max(
          -1,
          ...candidates.filter((d) => d.entity_type === f.entity_type).map((d) => d.position)
        ) + 1,
    };
    fields.push(newField);
    fieldMap.set(f.id, newField.id);
    changes.push({
      kind: "field",
      entity_type: f.entity_type,
      source_name: f.name,
      destination_name: name,
    });
  }
  const tagMap = new Map<string, string>();
  const tags: Tag[] = [];
  const visiting = new Set<string>();
  function mapTag(id: string): string {
    if (tagMap.has(id)) return tagMap.get(id)!;
    if (visiting.has(id)) throw new Error("Source tag hierarchy contains a cycle");
    visiting.add(id);
    const t = state.tags.find((t) => t.id === id && t.project_id === source!.id);
    if (!t) throw new Error("Missing source tag");
    const parent = t.parent_id ? mapTag(t.parent_id) : null;
    const candidates = [...state.tags.filter((d) => d.project_id === destination!.id), ...tags];
    const existing = candidates.filter(
      (d) => d.name === t.name && d.color === t.color && d.parent_id === parent
    );
    let target: string;
    if (existing.length === 1) target = existing[0].id;
    else {
      const used = candidates.map((d) => d.name);
      const name = used.some((n) => normalize(n) === normalize(t.name))
        ? available(t.name, `from ${source!.name}`, used)
        : t.name;
      const next = {
        ...t,
        id: crypto.randomUUID(),
        project_id: destination!.id,
        parent_id: parent,
        name,
        position: Math.max(-1, ...candidates.map((d) => d.position)) + 1,
      };
      tags.push(next);
      target = next.id;
      changes.push({ kind: "tag", entity_type: "", source_name: t.name, destination_name: name });
    }
    visiting.delete(id);
    tagMap.set(id, target);
    return target;
  }
  const assignments = state.assignments.filter((a) => ids.has(a.entity_id));
  assignments.forEach((a) => {
    const category = sourceRefs.find((r) => r.id === a.entity_id)!.reference_type;
    if (a.entity_type !== category && a.entity_type !== REFERENCE_FIELD_TYPES[category])
      throw new Error("Invalid tag assignment");
    mapTag(a.tag_id);
  });
  const values = state.values.filter((v) => ids.has(v.entity_id));
  if (values.some((v) => !fieldMap.has(v.field_definition_id)))
    throw new Error("Invalid source field");
  const revisionData = (project: Project) => {
    const references = state.references.filter((r) => r.project_id === project.id);
    const referenceIds = new Set(references.map((r) => r.id));
    return {
      project_id: project.id,
      reference_types: project.reference_types,
      references,
      fields: state.fields.filter((f) => f.project_id === project.id),
      values: state.values.filter((v) => referenceIds.has(v.entity_id)),
      tags: state.tags.filter((t) => t.project_id === project.id),
      assignments: state.assignments.filter((a) => referenceIds.has(a.entity_id)),
    };
  };
  const preview: ReferenceCopyPreview = {
    references,
    changes,
    enabled_types: [...new Set(rows.map((r) => r.reference_type))].filter(
      (t) => !destination.reference_types.includes(t)
    ),
    copied: rows.length,
    skipped: references.filter((r) => r.action === "skip").length,
    revision: JSON.stringify({
      source: revisionData(source),
      destination: revisionData(destination),
      request: { ...request, selection },
    }),
  };
  return { preview, sourceRefs, destination, fields, tags, values, assignments, fieldMap, tagMap };
}
