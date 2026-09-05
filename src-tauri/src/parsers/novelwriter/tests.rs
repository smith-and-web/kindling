use super::*;
use crate::{commands::insert_novelwriter, db};
use rusqlite::Connection;

pub(crate) fn fixture() -> (Connection, Project) {
    let conn = Connection::open_in_memory().unwrap();
    db::initialize_schema(&conn).unwrap();
    let mut project = Project::new("A & B".into(), SourceType::Blank, None);
    project.author_pen_name = Some("O'Brien".into());
    project.reference_types.push("items".into());
    db::insert_project(&conn, &project).unwrap();
    let part = Chapter::new(project.id, "Part I".into(), 0).with_is_part(true);
    db::insert_chapter(&conn, &part).unwrap();
    let chapter = Chapter::new(project.id, "Chapter \"One\"".into(), 1);
    db::insert_chapter(&conn, &chapter).unwrap();
    let scene = Scene::new(
        chapter.id,
        "Arrival".into(),
        Some("A stranger arrives.\nAt midnight.".into()),
        0,
    );
    db::insert_scene(&conn, &scene).unwrap();
    for (n, (title, text)) in [
        (
            "The knock",
            "<p>Someone <strong>knocked</strong> at the <em>door</em>.</p>",
        ),
        (
            "Answer",
            "<p><s>She ran.</s> She waited &amp; listened.</p>",
        ),
    ]
    .iter()
    .enumerate()
    {
        let mut beat = Beat::new(scene.id, title.to_string(), n as i32);
        beat.prose = Some(text.to_string());
        db::insert_beat(&conn, &beat).unwrap();
    }
    let mut character = Character::new(project.id, "Jane".into(), Some("The visitor".into()), None);
    character.attributes.insert("Age".into(), "30".into());
    character
        .attributes
        .insert("History".into(), "Born here.\nLeft once.\n".into());
    db::insert_character(&conn, &character).unwrap();
    db::add_scene_character_ref(&conn, &scene.id, &character.id).unwrap();
    let mut location = Location::new(project.id, "JANE".into(), Some("A house".into()), None);
    location.attributes.insert("Weather".into(), "Rain".into());
    db::insert_location(&conn, &location).unwrap();
    db::add_scene_location_ref(&conn, &scene.id, &location.id).unwrap();
    let mut object = ReferenceItem::new(
        project.id,
        "items".into(),
        "Key".into(),
        Some("An old key".into()),
        None,
    );
    object.attributes.insert("Metal".into(), "Iron".into());
    db::insert_reference_item(&conn, &object).unwrap();
    db::add_scene_reference_item_ref(&conn, &scene.id, &object.id).unwrap();
    (conn, project)
}
#[test]
fn handles_hash_and_documents() {
    assert!(is_handle(&new_handle()));
    assert!(!is_handle("../bad"));
    assert_eq!(sha1(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    assert_eq!(sha1(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
    let item = NwItem {
        handle: new_handle(),
        parent: new_handle(),
        root: new_handle(),
        order: 0,
        item_type: "FILE".into(),
        class: "NOVEL".into(),
        layout: "DOCUMENT".into(),
        heading: "H3".into(),
        name: "A \"quoted\" \\ name".into(),
        status: "s000000".into(),
    };
    let text = write_document(&item, "### Arrival\n\nHello **world**.\n");
    let parsed = read_document(&text).unwrap();
    assert_eq!(parsed.metadata["name"], item.name);
    assert_eq!(parsed.metadata["textHash"], sha1(parsed.body.as_bytes()));
    assert_eq!(write_document(&item, &parsed.body), text);
    assert!(read_document("+++\ninvalid").is_err());
    assert_eq!(
        read_document("%%~name: old\n%%~hash: old\n### Old\nText")
            .unwrap()
            .body,
        "### Old\nText"
    );
    assert!(matches!(
        read_nwx("<wrong/>"),
        Err(NovelWriterError::Invalid(_))
    ));
    assert!(
        matches!(read_nwx("<novelWriterXML fileVersion=\"9\"/>"), Err(NovelWriterError::Version(v)) if v == "9")
    );
}
#[test]
fn export_import_export_identity_and_references() {
    let (conn, project) = fixture();
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    export_novelwriter_project(&conn, &project.id, &first, &Default::default()).unwrap();
    let original =
        read_nwx(&std::fs::read_to_string(first.join("nwProject.nwx")).unwrap()).unwrap();
    assert_eq!(read_nwx(&write_nwx(&original)).unwrap(), original);
    let parsed = parse_novelwriter_project(&first).unwrap();
    assert_eq!(
        (
            parsed.chapters.len(),
            parsed.scenes.len(),
            parsed.beats.len()
        ),
        (2, 1, 2)
    );
    assert!(parsed.chapters[0].is_part);
    assert!(parsed.beats[0].prose.as_ref().unwrap().contains("<strong>"));
    assert_eq!(parsed.characters[0].attributes["Age"], "30");
    assert_eq!(
        parsed.characters[0].attributes["History"],
        "Born here.\nLeft once.\n"
    );
    assert_eq!(parsed.locations[0].attributes["Weather"], "Rain");
    assert_eq!(parsed.reference_items[0].attributes["Metal"], "Iron");
    assert_eq!(
        (
            parsed.scene_character_refs.len(),
            parsed.scene_location_refs.len(),
            parsed.scene_reference_item_refs.len()
        ),
        (1, 1, 1)
    );
    assert_eq!(
        parsed.scenes[0].synopsis.as_deref(),
        Some("A stranger arrives.\nAt midnight.")
    );
    let imported_conn = Connection::open_in_memory().unwrap();
    db::initialize_schema(&imported_conn).unwrap();
    insert_novelwriter(&imported_conn, &parsed).unwrap();
    export_novelwriter_project(
        &imported_conn,
        &parsed.project.id,
        &second,
        &Default::default(),
    )
    .unwrap();
    for entry in std::fs::read_dir(first.join("content")).unwrap() {
        let entry = entry.unwrap();
        assert_eq!(
            std::fs::read(entry.path()).unwrap(),
            std::fs::read(second.join("content").join(entry.file_name())).unwrap(),
            "{:?}",
            entry.file_name()
        );
    }
    let second_doc =
        read_nwx(&std::fs::read_to_string(second.join("nwProject.nwx")).unwrap()).unwrap();
    assert_eq!(original.items, second_doc.items);
    let reparsed = parse_novelwriter_project(&second).unwrap();
    assert_eq!(
        parsed
            .beats
            .iter()
            .map(|b| (&b.content, &b.prose, &b.source_id))
            .collect::<Vec<_>>(),
        reparsed
            .beats
            .iter()
            .map(|b| (&b.content, &b.prose, &b.source_id))
            .collect::<Vec<_>>()
    );
    assert!(
        export_novelwriter_project(&conn, &project.id, &first, &Default::default())
            .unwrap_err()
            .contains("empty")
    );
}
#[test]
fn options_archive_legacy_and_page() {
    let (conn, project) = fixture();
    let temp = tempfile::tempdir().unwrap();
    let opts = NovelWriterExportOptions {
        include_beat_comments: false,
        include_notes: false,
        create_snapshot: false,
    };
    export_novelwriter_project(&conn, &project.id, temp.path(), &opts).unwrap();
    let parsed = parse_novelwriter_project(temp.path()).unwrap();
    assert!(parsed.characters.is_empty());
    assert_eq!(parsed.beats.len(), 1);
    assert_eq!(parsed.beats[0].content, "Scene Content");
    assert!(parsed.scenes_with_beat_comments.is_empty());
    // Legacy extension and header are supported alongside modern imports.
    for file in std::fs::read_dir(temp.path().join("content")).unwrap() {
        let file = file.unwrap().path();
        let body = read_document(&std::fs::read_to_string(&file).unwrap())
            .unwrap()
            .body;
        std::fs::write(
            file.with_extension("nwd"),
            format!("%%~name: legacy\n{body}"),
        )
        .unwrap();
        std::fs::remove_file(file).unwrap();
    }
    assert_eq!(
        parse_novelwriter_project(temp.path()).unwrap().scenes.len(),
        1
    );
    let ch = db::get_chapters(&conn, &project.id).unwrap()[1].id;
    conn.execute(
        "UPDATE chapters SET archived = 1 WHERE id = ?1",
        [ch.to_string()],
    )
    .unwrap();
    let archive = temp.path().join("archived");
    export_novelwriter_project(&conn, &project.id, &archive, &Default::default()).unwrap();
    assert!(parse_novelwriter_project(&archive)
        .unwrap()
        .scenes
        .is_empty());
    assert!(parse_novelwriter_project(&temp.path().join("missing"))
        .unwrap_err()
        .to_string()
        .contains("nwProject.nwx"));
}
#[test]
fn foreign_sync_ids_are_preserved_and_exports_stable() {
    let (conn, project) = fixture();
    conn.execute(
        "UPDATE projects SET source_type = 'plottr' WHERE id = ?1",
        [project.id.to_string()],
    )
    .unwrap();
    conn.execute(
        "UPDATE chapters SET source_id = 'plottr-original' WHERE is_part = 0",
        [],
    )
    .unwrap();
    let temp = tempfile::tempdir().unwrap();
    for name in ["one", "two"] {
        export_novelwriter_project(
            &conn,
            &project.id,
            &temp.path().join(name),
            &Default::default(),
        )
        .unwrap();
    }
    assert_eq!(
        db::get_chapters(&conn, &project.id).unwrap()[1]
            .source_id
            .as_deref(),
        Some("plottr-original")
    );
    let a =
        read_nwx(&std::fs::read_to_string(temp.path().join("one/nwProject.nwx")).unwrap()).unwrap();
    let b =
        read_nwx(&std::fs::read_to_string(temp.path().join("two/nwProject.nwx")).unwrap()).unwrap();
    assert_eq!(a.items, b.items);
}

#[test]
fn page_export_identity_and_literal_document_syntax() {
    let (conn, project) = fixture();
    let temp = tempfile::tempdir().unwrap();
    conn.execute("UPDATE scenes SET editor_mode = 'page', prose = '<p># A literal heading</p><p>% Beat: literal comment</p><p>@char: not a reference</p>'", []).unwrap();
    let first = temp.path().join("first");
    export_novelwriter_project(&conn, &project.id, &first, &Default::default()).unwrap();
    let parsed = parse_novelwriter_project(&first).unwrap();
    assert_eq!(parsed.scenes.len(), 1);
    assert_eq!(parsed.scenes[0].editor_mode, EditorMode::Page);
    assert!(parsed.scenes[0]
        .prose
        .as_deref()
        .unwrap()
        .contains("% Beat: literal comment"));
    let other = Connection::open_in_memory().unwrap();
    db::initialize_schema(&other).unwrap();
    insert_novelwriter(&other, &parsed).unwrap();
    let second = temp.path().join("second");
    export_novelwriter_project(&other, &parsed.project.id, &second, &Default::default()).unwrap();
    for entry in std::fs::read_dir(first.join("content")).unwrap() {
        let e = entry.unwrap();
        assert_eq!(
            std::fs::read(e.path()).unwrap(),
            std::fs::read(second.join("content").join(e.file_name())).unwrap()
        );
    }
}
#[test]
fn root_exclusions_versions_and_invalid_trees() {
    let (conn, project) = fixture();
    let temp = tempfile::tempdir().unwrap();
    export_novelwriter_project(&conn, &project.id, temp.path(), &Default::default()).unwrap();
    let path = temp.path().join("nwProject.nwx");
    let text = std::fs::read_to_string(&path).unwrap();
    for version in ["1.4", "1.5", "1.6"] {
        assert!(read_nwx(
            &text.replace("fileVersion=\"1.6\"", &format!("fileVersion=\"{version}\""))
        )
        .is_ok());
    }
    for class in ["ARCHIVE", "TRASH", "TEMPLATE"] {
        std::fs::write(
            &path,
            text.replace("class=\"NOVEL\"", &format!("class=\"{class}\"")),
        )
        .unwrap();
        let parsed = parse_novelwriter_project(temp.path()).unwrap();
        assert!(parsed.scenes.is_empty());
        assert!(parsed.chapters.is_empty());
    }
    let mut doc = read_nwx(&text).unwrap();
    doc.items.push(doc.items[0].clone());
    assert!(read_nwx(&write_nwx(&doc)).is_err());
    doc.items.pop();
    let file = doc
        .items
        .iter_mut()
        .find(|i| i.item_type == "FILE")
        .unwrap();
    file.parent = file.handle.clone();
    assert!(read_nwx(&write_nwx(&doc)).is_err());
    assert_eq!(
        SourceType::parse(SourceType::NovelWriter.as_str()),
        Some(SourceType::NovelWriter)
    );
}

#[test]
fn malformed_quoted_headers_return_errors_instead_of_panicking() {
    for value in ["'", "'unclosed", "\"", "\"unclosed"] {
        assert!(matches!(
            read_document(&format!("+++\nname = {value}\n+++\nText")),
            Err(NovelWriterError::Invalid(_))
        ));
    }
    assert_eq!(
        read_document("+++\nname = ''\n+++\nText").unwrap().metadata["name"],
        ""
    );
}

#[test]
fn part_prose_and_following_scenes_have_reachable_chapters() {
    let (conn, project) = fixture();
    let temp = tempfile::tempdir().unwrap();
    export_novelwriter_project(&conn, &project.id, temp.path(), &Default::default()).unwrap();
    let index = temp.path().join("nwProject.nwx");
    let mut doc = read_nwx(&std::fs::read_to_string(&index).unwrap()).unwrap();
    let part = doc
        .items
        .iter()
        .find(|i| i.heading == "H1" && i.class == "NOVEL")
        .unwrap()
        .clone();
    let chapter = doc
        .items
        .iter()
        .find(|i| i.heading == "H2")
        .unwrap()
        .clone();
    for item in &mut doc.items {
        if item.parent == chapter.handle {
            item.parent = part.handle.clone();
        }
    }
    doc.items.retain(|i| i.handle != chapter.handle);
    std::fs::write(&index, write_nwx(&doc)).unwrap();
    let file = temp
        .path()
        .join("content")
        .join(format!("{}.md", part.handle));
    std::fs::write(&file, "# Part I\n\nPart opening prose.\n").unwrap();
    let parsed = parse_novelwriter_project(temp.path()).unwrap();
    assert_eq!(parsed.scenes.len(), 2);
    assert!(parsed.chapters[0].is_part);
    for scene in &parsed.scenes {
        assert!(
            !parsed
                .chapters
                .iter()
                .find(|c| c.id == scene.chapter_id)
                .unwrap()
                .is_part
        );
    }
    assert_eq!(parsed.scenes[0].chapter_id, parsed.scenes[1].chapter_id);
    assert!(parsed.scenes[0]
        .prose
        .as_deref()
        .unwrap()
        .contains("Part opening prose."));
    let repeat = parse_novelwriter_project(temp.path()).unwrap();
    assert_eq!(parsed.chapters[1].source_id, repeat.chapters[1].source_id);
}

#[test]
fn display_name_tags_plot_and_time_links_round_trip() {
    let (conn, project) = fixture();
    let scene = db::get_all_project_scenes(&conn, &project.id)
        .unwrap()
        .remove(0);
    for kind in ["objectives", "timelines", "custom", "organizations"] {
        let item = ReferenceItem::new(
            project.id,
            kind.into(),
            kind.into(),
            Some("Keep note".into()),
            None,
        );
        db::insert_reference_item(&conn, &item).unwrap();
        db::add_scene_reference_item_ref(&conn, &scene.id, &item.id).unwrap();
    }
    let temp = tempfile::tempdir().unwrap();
    export_novelwriter_project(&conn, &project.id, temp.path(), &Default::default()).unwrap();
    for entry in std::fs::read_dir(temp.path().join("content")).unwrap() {
        let file = entry.unwrap().path();
        let text = std::fs::read_to_string(&file).unwrap();
        std::fs::write(
            &file,
            text.replace("@tag: jane\n", "@tag: Jane | Jane Doe\n"),
        )
        .unwrap();
    }
    let parsed = parse_novelwriter_project(temp.path()).unwrap();
    assert_eq!(parsed.scene_character_refs.len(), 1);
    assert_eq!(parsed.scene_location_refs.len(), 1);
    assert_eq!(parsed.scene_reference_item_refs.len(), 5);
    let scene_file = temp.path().join("content").join(format!(
        "{}.md",
        parsed.scenes[0].source_id.as_ref().unwrap()
    ));
    let text = std::fs::read_to_string(scene_file).unwrap();
    assert!(text.contains("@plot: objectives"));
    assert!(text.contains("@time: timelines"));
}

#[test]
fn export_allows_os_metadata_but_rejects_hidden_user_content() {
    let (conn, project) = fixture();
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join(".DS_Store"), "finder metadata").unwrap();
    export_novelwriter_project(&conn, &project.id, temp.path(), &Default::default()).unwrap();
    assert_eq!(
        std::fs::read_to_string(temp.path().join(".DS_Store")).unwrap(),
        "finder metadata"
    );
    assert!(temp.path().join("nwProject.nwx").exists());
    for name in [".draft", ".git", "content"] {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join(name), "keep").unwrap();
        assert!(
            export_novelwriter_project(&conn, &project.id, temp.path(), &Default::default())
                .is_err()
        );
        assert_eq!(
            std::fs::read_to_string(temp.path().join(name)).unwrap(),
            "keep"
        );
    }
}
