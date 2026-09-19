//! Lossless, additive reference transfers within the local database.
use std::collections::{BTreeMap, BTreeSet, HashMap};

use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use uuid::Uuid;

use crate::models::{FieldDefinition, FieldValue, Project, Tag};

const TYPES: &[(&str, &str)] = &[
    ("characters", "character"),
    ("locations", "location"),
    ("items", "item"),
    ("objectives", "objective"),
    ("organizations", "organization"),
    ("timelines", "timeline"),
    ("custom", "custom"),
];
type Result<T> = std::result::Result<T, CopyError>;

#[derive(Debug, Serialize)]
pub struct CopyError {
    pub code: &'static str,
    pub message: String,
}
impl CopyError {
    fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_copy",
            message: message.into(),
        }
    }
}
impl From<rusqlite::Error> for CopyError {
    fn from(e: rusqlite::Error) -> Self {
        Self {
            code: "database_error",
            message: e.to_string(),
        }
    }
}
impl From<serde_json::Error> for CopyError {
    fn from(e: serde_json::Error) -> Self {
        Self::invalid(e.to_string())
    }
}
impl From<uuid::Error> for CopyError {
    fn from(e: uuid::Error) -> Self {
        Self::invalid(e.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReferenceKey {
    pub reference_type: String,
    pub id: Uuid,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CopyRequest {
    pub source_project_id: Uuid,
    pub destination_project_id: Uuid,
    pub selection: Option<Vec<ReferenceKey>>,
    #[serde(default)]
    pub keep_both: Vec<ReferenceKey>,
}
#[derive(Debug, Clone, Serialize)]
pub struct CopyRow {
    #[serde(flatten)]
    pub key: ReferenceKey,
    pub name: String,
    pub description: Option<String>,
    pub selected: bool,
    pub conflict: bool,
    pub action: String,
    pub destination_name: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct DependencyChange {
    pub kind: String,
    pub entity_type: String,
    pub source_name: String,
    pub destination_name: String,
}
#[derive(Debug, Serialize)]
pub struct CopyPreview {
    pub references: Vec<CopyRow>,
    pub changes: Vec<DependencyChange>,
    pub enabled_types: Vec<String>,
    pub copied: usize,
    pub skipped: usize,
    pub revision: String,
}
#[derive(Debug, Serialize)]
pub struct CopyResult {
    pub project: Project,
    pub created_reference_ids: Vec<Uuid>,
    pub copied: usize,
    pub skipped: usize,
}
#[derive(Debug, Clone, Serialize)]
struct Reference {
    key: ReferenceKey,
    name: String,
    description: Option<String>,
    source_id: Option<String>,
    // Legacy tables permit null values; preserve them rather than silently discarding them.
    attributes: BTreeMap<String, Option<String>>,
}
#[derive(Debug, Clone, Serialize)]
struct Assignment {
    tag_id: Uuid,
    entity_type: String,
    entity_id: Uuid,
}
#[derive(Debug, Serialize)]
struct Library {
    project: Project,
    references: Vec<Reference>,
    fields: Vec<FieldDefinition>,
    values: Vec<FieldValue>,
    tags: Vec<Tag>,
    assignments: Vec<Assignment>,
}

// Project prose saves and metadata edits cannot change reference-copy decisions.
// Keep only the project identity/settings used by the transfer, plus its library.
fn revision_data(library: &Library) -> serde_json::Value {
    serde_json::json!({
        "project_id": library.project.id,
        "reference_types": library.project.reference_types,
        "references": library.references,
        "fields": library.fields,
        "values": library.values,
        "tags": library.tags,
        "assignments": library.assignments,
    })
}

fn entity_type(category: &str) -> Result<&'static str> {
    TYPES
        .iter()
        .find(|(t, _)| *t == category)
        .map(|(_, e)| *e)
        .ok_or_else(|| CopyError::invalid(format!("Unsupported reference category: {category}")))
}
fn id(row: &Row<'_>, i: usize) -> rusqlite::Result<Uuid> {
    let s: String = row.get(i)?;
    Uuid::parse_str(&s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(i, rusqlite::types::Type::Text, Box::new(e))
    })
}
fn read<T>(
    conn: &Connection,
    sql: &str,
    project: Uuid,
    f: impl FnMut(&Row<'_>) -> rusqlite::Result<T>,
) -> Result<Vec<T>> {
    Ok(conn
        .prepare(sql)?
        .query_map([project.to_string()], f)?
        .collect::<rusqlite::Result<Vec<_>>>()?)
}

// Fixed internal identifiers only; no SQL identifiers come from the IPC request.
fn storage(category: &str) -> (&'static str, &'static str, &'static str) {
    match category {
        "characters" => ("characters", "character_attributes", "character_id"),
        "locations" => ("locations", "location_attributes", "location_id"),
        _ => (
            "reference_items",
            "reference_item_attributes",
            "reference_item_id",
        ),
    }
}

fn library(conn: &Connection, project_id: Uuid) -> Result<Library> {
    let project = super::get_project(conn, &project_id)?
        .ok_or_else(|| CopyError::invalid("Project not found"))?;
    let mut references = Vec::new();
    for category in ["characters", "locations", "generic"] {
        let (table, attrs, attr_id) = storage(category);
        let type_expr = match category {
            "characters" => "'characters'",
            "locations" => "'locations'",
            _ => "reference_type",
        };
        let mut records = read(conn, &format!("SELECT id, name, description, source_id, {type_expr} FROM {table} WHERE project_id = ?1 ORDER BY id"), project_id, |r| {
            Ok(Reference { key: ReferenceKey { id: id(r, 0)?, reference_type: r.get(4)? }, name: r.get(1)?, description: r.get(2)?, source_id: r.get(3)?, attributes: BTreeMap::new() })
        })?;
        let attributes = read(conn, &format!("SELECT a.{attr_id}, a.key, a.value FROM {attrs} a JOIN {table} r ON r.id = a.{attr_id} WHERE r.project_id = ?1 ORDER BY a.{attr_id}, a.key"), project_id,
            |r| Ok((id(r, 0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?)))?;
        let mut attr_map: HashMap<Uuid, BTreeMap<String, Option<String>>> = HashMap::new();
        for (id, key, value) in attributes {
            attr_map.entry(id).or_default().insert(key, value);
        }
        for record in &mut records {
            entity_type(&record.key.reference_type)?;
            if category == "generic"
                && matches!(
                    record.key.reference_type.as_str(),
                    "characters" | "locations"
                )
            {
                return Err(CopyError::invalid("Reference is stored in the wrong table"));
            }
            record.attributes = attr_map.remove(&record.key.id).unwrap_or_default();
        }
        references.extend(records);
    }
    references.sort_by_key(|r| (r.key.reference_type.clone(), normalized(&r.name), r.key.id));
    let reference_ids: BTreeSet<_> = references.iter().map(|r| r.key.id).collect();
    if reference_ids.len() != references.len() {
        return Err(CopyError::invalid("Ambiguous reference IDs"));
    }
    let fields = read(conn, "SELECT id, project_id, entity_type, name, field_type, options, default_value, position, required, visible, created_at FROM field_definitions WHERE project_id = ?1 ORDER BY entity_type, position, id", project_id, |r| {
        Ok(FieldDefinition { id: id(r,0)?, project_id: id(r,1)?, entity_type:r.get(2)?, name:r.get(3)?, field_type:r.get(4)?, options:r.get(5)?, default_value:r.get(6)?, position:r.get(7)?, required:r.get(8)?, visible:r.get(9)?, created_at:r.get(10)? })
    })?;
    // The CTE includes only references, never scenes or beats, and uses one parameter.
    let entities = "WITH entities AS (SELECT id FROM characters WHERE project_id = ?1 UNION SELECT id FROM locations WHERE project_id = ?1 UNION SELECT id FROM reference_items WHERE project_id = ?1)";
    let values = read(conn, &format!("{entities} SELECT id, field_definition_id, entity_id, value FROM field_values WHERE entity_id IN (SELECT id FROM entities) ORDER BY entity_id, field_definition_id, id"), project_id, |r| {
        Ok(FieldValue { id:id(r,0)?, field_definition_id:id(r,1)?, entity_id:id(r,2)?, value:r.get(3)? })
    })?;
    let tags = read(conn, "SELECT id, project_id, name, color, parent_id, position, created_at FROM tags WHERE project_id = ?1 ORDER BY position, id", project_id, |r| {
        Ok(Tag { id:id(r,0)?, project_id:id(r,1)?, name:r.get(2)?, color:r.get(3)?, parent_id:r.get::<_, Option<String>>(4)?.map(|_| id(r,4)).transpose()?, position:r.get(5)?, created_at:r.get(6)? })
    })?;
    let assignments = read(conn, &format!("{entities} SELECT tag_id, entity_type, entity_id FROM entity_tags WHERE entity_id IN (SELECT id FROM entities) ORDER BY entity_id, tag_id, entity_type"), project_id,
        |r| Ok(Assignment { tag_id:id(r,0)?, entity_type:r.get(1)?, entity_id:id(r,2)? }))?;
    Ok(Library {
        project,
        references,
        fields,
        values,
        tags,
        assignments,
    })
}
fn normalized(name: &str) -> String {
    name.trim().to_lowercase()
}
fn available(name: &str, suffix: &str, used: &BTreeSet<String>) -> String {
    let mut n = 1;
    loop {
        let candidate = if n == 1 {
            format!("{name} ({suffix})")
        } else {
            format!("{name} ({suffix} {n})")
        };
        if !used.contains(&normalized(&candidate)) {
            return candidate;
        }
        n += 1;
    }
}
fn compatible(a: &FieldDefinition, b: &FieldDefinition) -> Result<bool> {
    let options = |v: &Option<String>| -> Result<Option<serde_json::Value>> {
        Ok(v.as_deref().map(serde_json::from_str).transpose()?)
    };
    Ok(a.entity_type == b.entity_type
        && a.name == b.name
        && a.field_type == b.field_type
        && options(&a.options)? == options(&b.options)?
        && a.default_value == b.default_value
        && a.required == b.required
        && a.visible == b.visible)
}
fn next_position(positions: impl Iterator<Item = i32>) -> Result<i32> {
    positions
        .max()
        .unwrap_or(-1)
        .checked_add(1)
        .ok_or_else(|| CopyError::invalid("No available position"))
}

struct Plan {
    source: Library,
    destination: Library,
    preview: CopyPreview,
    fields: Vec<FieldDefinition>,
    field_map: HashMap<Uuid, Uuid>,
    tags: Vec<Tag>,
    tag_map: HashMap<Uuid, Uuid>,
}

fn plan(conn: &Connection, request: &CopyRequest) -> Result<Plan> {
    if request.source_project_id == request.destination_project_id {
        return Err(CopyError::invalid("Choose a different source project"));
    }
    let source = library(conn, request.source_project_id)?;
    let destination = library(conn, request.destination_project_id)?;
    let all: BTreeSet<_> = source.references.iter().map(|r| r.key.clone()).collect();
    let selection: BTreeSet<_> = request
        .selection
        .as_ref()
        .map(|s| s.iter().cloned().collect())
        .unwrap_or_else(|| all.clone());
    if request
        .selection
        .as_ref()
        .is_some_and(|s| s.len() != selection.len())
        || !selection.is_subset(&all)
    {
        return Err(CopyError::invalid(
            "Selection contains duplicate, missing, or reclassified references",
        ));
    }
    let keep: BTreeSet<_> = request.keep_both.iter().cloned().collect();
    if keep.len() != request.keep_both.len() || !keep.is_subset(&selection) {
        return Err(CopyError::invalid("Invalid keep-both choices"));
    }
    // Value canonicalizes nested maps (Reference attributes are already ordered). Selection
    // None and explicit 'all' have the same revision so the initial preview is committable.
    let canonical = serde_json::to_value((
        revision_data(&source),
        revision_data(&destination),
        &selection,
        &keep,
    ))?;
    let revision = format!("{:x}", Sha1::digest(serde_json::to_vec(&canonical)?));
    let mut names: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for r in &destination.references {
        names
            .entry(r.key.reference_type.clone())
            .or_default()
            .insert(normalized(&r.name));
    }
    let destination_names = names.clone();
    let mut rows = Vec::new();
    for r in &source.references {
        let used = names.entry(r.key.reference_type.clone()).or_default();
        let selected = selection.contains(&r.key);
        let conflict = destination_names
            .get(&r.key.reference_type)
            .is_some_and(|names| names.contains(&normalized(&r.name)));
        let copying = selected && (!conflict || keep.contains(&r.key));
        let name = if copying && used.contains(&normalized(&r.name)) {
            available(&r.name, "copy", used)
        } else {
            r.name.clone()
        };
        if copying {
            used.insert(normalized(&name));
        }
        rows.push(CopyRow {
            key: r.key.clone(),
            name: r.name.clone(),
            description: r.description.clone(),
            selected,
            conflict,
            action: if copying {
                "copy"
            } else if selected {
                "skip"
            } else {
                "unselected"
            }
            .into(),
            destination_name: name,
        });
    }
    let copied = rows.iter().filter(|r| r.action == "copy").count();
    let skipped = rows.iter().filter(|r| r.action == "skip").count();
    let enabled_types = TYPES
        .iter()
        .filter(|(t, _)| {
            rows.iter()
                .any(|r| r.action == "copy" && r.key.reference_type == *t)
                && !destination.project.reference_types.iter().any(|e| e == t)
        })
        .map(|(t, _)| t.to_string())
        .collect();
    let preview = CopyPreview {
        references: rows,
        changes: Vec::new(),
        enabled_types,
        copied,
        skipped,
        revision,
    };
    let mut plan = Plan {
        source,
        destination,
        preview,
        fields: Vec::new(),
        field_map: HashMap::new(),
        tags: Vec::new(),
        tag_map: HashMap::new(),
    };
    plan.dependencies()?;
    Ok(plan)
}
impl Plan {
    fn dependencies(&mut self) -> Result<()> {
        let copying: HashMap<_, _> = self
            .preview
            .references
            .iter()
            .filter(|r| r.action == "copy")
            .map(|r| Ok((r.key.id, entity_type(&r.key.reference_type)?)))
            .collect::<Result<_>>()?;
        let types: BTreeSet<_> = copying.values().copied().collect();
        let mut candidates = self.destination.fields.clone();
        for field in self
            .source
            .fields
            .iter()
            .filter(|f| types.contains(f.entity_type.as_str()))
        {
            // Validate even options without a matching destination definition.
            if let Some(options) = &field.options {
                serde_json::from_str::<serde_json::Value>(options)?;
            }
            let matches = candidates
                .iter()
                .filter_map(|d| match compatible(field, d) {
                    Ok(true) => Some(Ok(d.id)),
                    Ok(false) => None,
                    Err(e) => Some(Err(e)),
                })
                .collect::<Result<Vec<_>>>()?;
            let target =
                if matches.len() == 1 && !self.field_map.values().any(|id| *id == matches[0]) {
                    matches[0]
                } else {
                    let used: BTreeSet<_> = candidates
                        .iter()
                        .filter(|f| f.entity_type == field.entity_type)
                        .map(|f| normalized(&f.name))
                        .collect();
                    let mut new = field.clone();
                    new.id = Uuid::new_v4();
                    new.project_id = self.destination.project.id;
                    new.created_at = chrono::Utc::now().to_rfc3339();
                    if used.contains(&normalized(&new.name)) {
                        new.name = available(
                            &new.name,
                            &format!("from {}", self.source.project.name),
                            &used,
                        );
                    }
                    new.position = next_position(
                        candidates
                            .iter()
                            .filter(|f| f.entity_type == new.entity_type)
                            .map(|f| f.position),
                    )?;
                    self.preview.changes.push(DependencyChange {
                        kind: "field".into(),
                        entity_type: new.entity_type.clone(),
                        source_name: field.name.clone(),
                        destination_name: new.name.clone(),
                    });
                    let id = new.id;
                    candidates.push(new.clone());
                    self.fields.push(new);
                    id
                };
            self.field_map.insert(field.id, target);
        }
        let fields: HashMap<_, _> = self.source.fields.iter().map(|f| (f.id, f)).collect();
        for value in &self.source.values {
            if let Some(entity) = copying.get(&value.entity_id) {
                if fields
                    .get(&value.field_definition_id)
                    .is_none_or(|f| &f.entity_type != entity)
                {
                    return Err(CopyError::invalid(
                        "A selected reference has a field from another project or category",
                    ));
                }
            }
        }
        let mut needed = BTreeSet::new();
        for a in &self.source.assignments {
            if let Some(entity) = copying.get(&a.entity_id) {
                // Existing TagSelector versions wrote category IDs (plural). Accept
                // both spellings here, preserving each stored assignment on insert.
                if &a.entity_type != entity && entity_type(&a.entity_type).ok() != Some(*entity) {
                    return Err(CopyError::invalid(
                        "A selected reference has an invalid tag assignment",
                    ));
                }
                needed.insert(a.tag_id);
            }
        }
        for id in needed {
            self.map_tag(id, &mut BTreeSet::new())?;
        }
        Ok(())
    }
    fn map_tag(&mut self, id: Uuid, visiting: &mut BTreeSet<Uuid>) -> Result<Uuid> {
        if let Some(mapped) = self.tag_map.get(&id) {
            return Ok(*mapped);
        }
        if !visiting.insert(id) {
            return Err(CopyError::invalid("Source tag hierarchy contains a cycle"));
        }
        let tag = self
            .source
            .tags
            .iter()
            .find(|t| t.id == id)
            .cloned()
            .ok_or_else(|| {
                CopyError::invalid(
                    "A selected tag or ancestor belongs to another project or is missing",
                )
            })?;
        let parent = tag
            .parent_id
            .map(|p| self.map_tag(p, visiting))
            .transpose()?;
        visiting.remove(&id);
        let candidates: Vec<_> = self
            .destination
            .tags
            .iter()
            .chain(&self.tags)
            .filter(|t| t.name == tag.name && t.color == tag.color && t.parent_id == parent)
            .collect();
        let mapped = if candidates.len() == 1 {
            candidates[0].id
        } else {
            // SQLite enforces UNIQUE(project_id, name), including across different parents.
            let used: BTreeSet<String> = self
                .destination
                .tags
                .iter()
                .chain(&self.tags)
                .map(|t| normalized(&t.name))
                .collect();
            let mut new = tag.clone();
            new.id = Uuid::new_v4();
            new.project_id = self.destination.project.id;
            new.parent_id = parent;
            new.created_at = chrono::Utc::now().to_rfc3339();
            if used.contains(&normalized(&new.name)) {
                new.name = available(
                    &new.name,
                    &format!("from {}", self.source.project.name),
                    &used,
                );
            }
            new.position = next_position(
                self.destination
                    .tags
                    .iter()
                    .chain(&self.tags)
                    .map(|t| t.position),
            )?;
            self.preview.changes.push(DependencyChange {
                kind: "tag".into(),
                entity_type: String::new(),
                source_name: tag.name,
                destination_name: new.name.clone(),
            });
            let new_id = new.id;
            self.tags.push(new);
            new_id
        };
        self.tag_map.insert(id, mapped);
        Ok(mapped)
    }
}

pub fn preview(conn: &Connection, request: &CopyRequest) -> Result<CopyPreview> {
    let tx = conn.unchecked_transaction()?;
    Ok(plan(&tx, request)?.preview)
}
pub fn copy(
    conn: &Connection,
    request: &CopyRequest,
    expected_revision: &str,
) -> Result<CopyResult> {
    if request.selection.is_none() {
        return Err(CopyError::invalid("Commit requires an explicit selection"));
    }
    let tx = conn.unchecked_transaction()?;
    let mut plan = plan(&tx, request)?;
    if plan.preview.revision != expected_revision {
        return Err(CopyError {
            code: "stale_preview",
            message: "References changed. Refresh the preview and review it before copying.".into(),
        });
    }
    for field in &plan.fields {
        super::create_field_definition(&tx, field)?;
    }
    for tag in &plan.tags {
        super::create_tag(&tx, tag)?;
    }
    let mut reference_map = HashMap::new();
    let sources: HashMap<_, _> = plan.source.references.iter().map(|r| (&r.key, r)).collect();
    let mut created_reference_ids = Vec::new();
    for row in plan
        .preview
        .references
        .iter()
        .filter(|r| r.action == "copy")
    {
        let original = sources[&row.key];
        let new_id = Uuid::new_v4();
        let (table, attrs, attr_id) = storage(&row.key.reference_type);
        if table == "reference_items" {
            tx.execute("INSERT INTO reference_items (id, project_id, reference_type, name, description, source_id) VALUES (?1, ?2, ?3, ?4, ?5, NULL)", params![new_id.to_string(), plan.destination.project.id.to_string(), row.key.reference_type, row.destination_name, original.description])?;
        } else {
            tx.execute(&format!("INSERT INTO {table} (id, project_id, name, description, source_id) VALUES (?1, ?2, ?3, ?4, NULL)"), params![new_id.to_string(), plan.destination.project.id.to_string(), row.destination_name, original.description])?;
        }
        for (key, value) in &original.attributes {
            tx.execute(
                &format!("INSERT INTO {attrs} ({attr_id}, key, value) VALUES (?1, ?2, ?3)"),
                params![new_id.to_string(), key, value],
            )?;
        }
        reference_map.insert(row.key.id, new_id);
        created_reference_ids.push(new_id);
    }
    for value in &plan.source.values {
        if let Some(entity) = reference_map.get(&value.entity_id) {
            super::set_field_value(
                &tx,
                &plan.field_map[&value.field_definition_id],
                entity,
                value.value.as_deref(),
            )?;
        }
    }
    for a in &plan.source.assignments {
        if let Some(entity) = reference_map.get(&a.entity_id) {
            super::tag_entity(&tx, &plan.tag_map[&a.tag_id], &a.entity_type, entity)?;
        }
    }
    if !created_reference_ids.is_empty() {
        plan.destination
            .project
            .reference_types
            .extend(plan.preview.enabled_types);
        plan.destination.project.modified_at = chrono::Utc::now().to_rfc3339();
        super::update_project(&tx, &plan.destination.project)?;
    }
    tx.commit()?;
    Ok(CopyResult {
        project: plan.destination.project,
        created_reference_ids,
        copied: plan.preview.copied,
        skipped: plan.preview.skipped,
    })
}

#[cfg(test)]
mod tests;
