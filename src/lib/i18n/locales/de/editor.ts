import type { Catalog } from "../../types";

/** Scene panel, beat list, prose editors and writing statistics. */
const catalog: Catalog = {
  // ScenePanel
  "scenePanel.revisions": "Versionsverlauf",
  "scenePanel.openingRevisions": "Versionsverlauf wird geöffnet…",
  "scenePanel.revisionsUnsavedError":
    "Bitte nicht gespeicherten Text speichern oder wiederherstellen, bevor Sie den Versionsverlauf öffnen.",
  "scenePanel.viewGroupLabel": "Ansicht",
  "scenePanel.beatsTab": "Beats",
  "scenePanel.beatsTabTooltip": "Beat-Karten",
  "scenePanel.pageTab": "Seite",
  "scenePanel.pageTabTooltip": "Seitentext",
  "scenePanel.sequence": "Sequenz",
  "scenePanel.chapter": "Kapitel",
  "scenePanel.breadcrumb": "{kind} · {name}",
  "scenePanel.pageEstimate": "~{pages} Seiten",
  "scenePanel.locked": "Gesperrt",
  "scenePanel.sceneTypeLabel": "Szenentyp",
  "scenePanel.sceneTypeNormal": "Normal",
  "scenePanel.sceneTypeNotes": "Notizen",
  "scenePanel.sceneTypeTodo": "Aufgabe",
  "scenePanel.sceneTypeUnused": "Ungenutzt",
  "scenePanel.statusLabel": "Status",
  "scenePanel.sceneStatusDraft": "Entwurf",
  "scenePanel.sceneStatusRevised": "Überarbeitet",
  "scenePanel.sceneStatusFinal": "Final",
  "scenePanel.planningLabel": "Planung",
  "scenePanel.planningFixed": "Fest",
  "scenePanel.planningFlexible": "Flexibel",
  "scenePanel.planningUndefined": "Undefiniert",
  "scenePanel.planningTooltip":
    "Legt fest, wie viel Struktur diese Szene hat: Undefiniert → Flexibel → Fest",
  "scenePanel.aboutPlanning": "Über Planung",
  "scenePanel.planningHelp":
    "Legt fest, wie viel Struktur diese Szene hat: Undefiniert, Flexibel oder Fest.",
  "scenePanel.tagsLabel": "Tags",
  "scenePanel.rollingOutline": "Fortlaufende Gliederung",
  "scenePanel.dismiss": "Schließen",
  "scenePanel.guideLedeBefore": "Das ",
  "scenePanel.guidePlanning": "Planungs",
  "scenePanel.guideLedeAfter":
    "-Menü oben legt fest, wie viel Struktur diese Szene hat. Damit können Sie Ihre Geschichte schrittweise ausarbeiten:",
  "scenePanel.guideUndefinedDesc":
    "Ein Platzhalter — Sie wissen, dass sie existiert, haben sie aber noch nicht geplant.",
  "scenePanel.guideFlexibleDesc":
    "Sie haben die Grundidee — eine Zusammenfassung und grobe Richtung.",
  "scenePanel.guideFixedDesc": "Vollständige Struktur mit Beats, Referenzen und Notizen.",
  "scenePanel.undefinedSceneTitle": "Undefinierte Szene",
  "scenePanel.undefinedSceneBody":
    "Dies ist ein Platzhalter — Sie wissen, dass sie existiert, haben sie aber noch nicht geplant.",
  "scenePanel.undefinedSceneHelp":
    "Fügen Sie unten eine Zusammenfassung hinzu, um die Grundidee festzuhalten, und stufen Sie sie dann hoch, wenn Sie bereit sind, sie auszuarbeiten.",
  "scenePanel.switchToFlexible": "Zu Flexibel wechseln",
  "scenePanel.goStraightToFixed": "Direkt zu Fest wechseln",
  "scenePanel.flexibleSceneTitle": "Flexible Szene",
  "scenePanel.flexibleSceneBody":
    "Sie haben eine Idee für diese Szene, haben die Struktur aber noch nicht festgelegt.",
  "scenePanel.flexibleSceneHelp":
    "Nutzen Sie die Zusammenfassung, um Ihre Absicht festzuhalten. Wenn Sie bereit sind, sie in Beats aufzuteilen, wechseln Sie zu Fest.",
  "scenePanel.defineBeats": "Beats festlegen",
  "scenePanel.sceneLockedTitle": "Diese Szene ist gesperrt",
  "scenePanel.chapterLockedHelp":
    "Das übergeordnete Kapitel ist gesperrt. Entsperren Sie das Kapitel, um diese Szene zu bearbeiten.",
  "scenePanel.sceneLockedHelp":
    "Entsperren Sie diese Szene in der Seitenleiste, um Änderungen vorzunehmen.",
  "scenePanel.synopsisTitle": "Zusammenfassung",
  "scenePanel.editSynopsis": "Zusammenfassung bearbeiten",
  "scenePanel.synopsisPlaceholder": "Kurze Zusammenfassung für diese Szene verfassen…",
  "scenePanel.saving": "Wird gespeichert…",
  "scenePanel.synopsisEditingHelp":
    "Escape drücken zum Schließen. Änderungen werden automatisch gespeichert.",
  "scenePanel.addSynopsis": "Zusammenfassung hinzufügen",
  "scenePanel.sceneLockedShort": "Szene ist gesperrt",
  "scenePanel.synopsisSaveFailed":
    "Zusammenfassung nicht gespeichert: {error}. Ihr Entwurf wird für erneuten Versuch behalten.",
  "scenePanel.retrySynopsisSave": "Zusammenfassungsspeicherung erneut versuchen",
  "scenePanel.retrySaving": "Erneut versuchen",
  "scenePanel.unsavedChanges": "Nicht gespeicherte Änderungen",
  "scenePanel.referencesTitle": "Referenzen",
  "scenePanel.loading": "Wird geladen…",
  "scenePanel.noReferences":
    "Noch keine Referenzen verknüpft. Verknüpfen Sie sie über den Referenzbereich.",
  "scenePanel.discoveryNotesTitle": "Entdeckungsnotizen",
  "scenePanel.hide": "Ausblenden",
  "scenePanel.show": "Einblenden",
  "scenePanel.newNoteLabel": "Neue Entdeckungsnotiz",
  "scenePanel.newNotePlaceholder": "Was haben Sie entdeckt?",
  "scenePanel.addingNote": "Wird hinzugefügt…",
  "scenePanel.addNote": "Notiz hinzufügen",
  "scenePanel.cancel": "Abbrechen",
  "scenePanel.editNoteLabel": "Entdeckungsnotiz bearbeiten",
  "scenePanel.save": "Speichern",
  "scenePanel.edit": "Bearbeiten",
  "scenePanel.promoting": "Wird hochgestuft…",
  "scenePanel.promoteToBeat": "Zu Beat hochstufen",
  "scenePanel.delete": "Löschen",
  "scenePanel.noNotes":
    "Noch keine Entdeckungsnotizen. Halten Sie fest, was Sie beim Schreiben entdecken — Notizen sind nicht Teil des Manuskripts.",
  "scenePanel.contentTitle": "Inhalt",
  "scenePanel.referenceLoadError": "Szenenreferenzen konnten nicht geladen werden",
  "scenePanel.metadataUpdateError": "Szenemetadaten konnten nicht aktualisiert werden",
  "scenePanel.planningUpdateError": "Planungsstatus konnte nicht aktualisiert werden",
  "scenePanel.renameSceneError": "Szene konnte nicht umbenannt werden: {error}",
  "scenePanel.addNoteError": "Entdeckungsnotiz konnte nicht hinzugefügt werden: {error}",
  "scenePanel.saveNoteError": "Entdeckungsnotiz konnte nicht gespeichert werden: {error}",
  "scenePanel.deleteNoteError": "Entdeckungsnotiz konnte nicht gelöscht werden: {error}",
  "scenePanel.promoteNoteError": "Notiz konnte nicht in einen Beat umgewandelt werden: {error}",
  "scenePanel.modeSwitchUnsavedError":
    "Bitte nicht gespeicherten Text in Suchen und Ersetzen speichern, wiederherstellen oder verwerfen, bevor Sie den Editormodus wechseln.",
  "scenePanel.modeSwitchError": "Editormodus konnte nicht gewechselt werden: {error}",
  "scenePanel.sceneChangedError":
    "Das ausgewählte Projekt oder die Szene hat sich geändert. Suchen und Ersetzen neu öffnen.",
  "scenePanel.emptyTitle": "Szene auswählen, um mit dem Schreiben zu beginnen",
  "scenePanel.emptyBody":
    "Wählen Sie eine Szene aus der Seitenleiste aus, um ihren Inhalt anzuzeigen.",
  "scenePanel.switchToBeatTitle": "Zur Beat-Ansicht wechseln",
  "scenePanel.switchToBeatMessage":
    "Alle in der Seitenansicht vorgenommenen Änderungen werden mit den entsprechenden Beats synchronisiert. Fortfahren?",
  "scenePanel.switchConfirm": "Wechseln",

  // BeatView
  "beatView.beatsTitle": "Beats",
  "beatView.addBeat": "Beat hinzufügen",
  "beatView.dragToReorder": "Zum Neuanordnen ziehen",
  "beatView.beatTitleLabel": "Beat-Titel",
  "beatView.beatWords.one": "{count} Wort",
  "beatView.beatWords.other": "{count} Wörter",
  "beatView.beatMenu": "Beat-Menü",
  "beatView.sceneLocked": "Szene ist gesperrt",
  "beatView.prosePlaceholder": "Text für diesen Beat verfassen…",
  "beatView.openBeatProse": "Text von Beat {number} öffnen",
  "beatView.emptyTitle": "Noch keine Beats",
  "beatView.emptyBody":
    "Beats kommen aus einer importierten Gliederung, oder Sie können sie hier einzeln hinzufügen.",
  "beatView.addFirstBeat": "Ersten Beat hinzufügen",
  "beatView.newBeatLabel": "Neuer Beat",
  "beatView.newBeatPlaceholder": "Beschreiben Sie, was in diesem Beat passiert…",
  "beatView.newBeatHelp": "Enter drücken zum Erstellen, Escape zum Abbrechen.",
  "beatView.cancel": "Abbrechen",
  "beatView.creating": "Wird erstellt…",
  "beatView.createBeat": "Beat erstellen",
  "beatView.deleteTitle": "Beat löschen",
  "beatView.deleteFirstMessage":
    "Diesen Beat wirklich löschen? Dies ist der erste Beat, daher wird sein Text dauerhaft gelöscht.",
  "beatView.deleteMergeMessage":
    "Diesen Beat wirklich löschen? Jeglicher Text wird in den vorherigen Beat zusammengeführt.",
  "beatView.menuRename": "Umbenennen",
  "beatView.menuSplit": "Am Cursor aufteilen",
  "beatView.menuMerge": "Mit nächstem zusammenführen",
  "beatView.menuMoveUp": "Nach oben",
  "beatView.menuMoveDown": "Nach unten",
  "beatView.menuDelete": "Löschen",
  "beatView.createError": "Beat konnte nicht erstellt werden: {error}",
  "beatView.renameError": "Beat konnte nicht umbenannt werden: {error}",
  "beatView.deleteError": "Beat konnte nicht gelöscht werden: {error}",
  "beatView.splitError": "Beat konnte nicht aufgeteilt werden: {error}",
  "beatView.mergeError": "Beats konnten nicht zusammengeführt werden: {error}",
  "beatView.transformUnsavedError":
    "Bitte zuerst die betroffenen Beats speichern oder ihre nicht gespeicherten Entwürfe in Suchen und Ersetzen wiederherstellen.",
  "beatView.reorderError": "Beats konnten nicht neu angeordnet werden: {error}",
  "beatView.moveUpAnnouncement":
    "Beat „{title}“ nach oben verschoben, an Position {position} von {total}.",
  "beatView.moveDownAnnouncement":
    "Beat „{title}“ nach unten verschoben, an Position {position} von {total}.",

  // ProseToolbar
  "proseToolbar.label": "Textformatierung",
  "proseToolbar.bold": "Fett",
  "proseToolbar.italic": "Kursiv",
  "proseToolbar.underline": "Unterstrichen",
  "proseToolbar.monospace": "Nichtproportional",
  "proseToolbar.alignLeft": "Linksbündig",
  "proseToolbar.alignCenter": "Zentriert",
  "proseToolbar.alignRight": "Rechtsbündig",
  "proseToolbar.alignJustify": "Blocksatz",
  "proseToolbar.blockquote": "Blockzitat",
  "proseToolbar.indent": "Einrücken",
  "proseToolbar.undo": "Rückgängig",
  "proseToolbar.redo": "Wiederholen",

  // SluglineInput
  "sluglineInput.sceneHeading": "Szenenüberschrift",
  "sluglineInput.interiorOrExterior": "Innen oder außen",
  "sluglineInput.location": "Ort",
  "sluglineInput.locationPlaceholder": "Ort",
  "sluglineInput.timeOfDay": "Tageszeit",

  // WritingStatusBar
  "writingStatusBar.statisticsTitle": "Schreibstatistiken",
  "writingStatusBar.hideStatistics": "Statistiken ausblenden",
  "writingStatusBar.totalWords": "Wörter gesamt",
  "writingStatusBar.scenesWithProse": "Szenen mit Text",
  "writingStatusBar.emptyScenes": "Leere Szenen",
  "writingStatusBar.averageWordsPerScene": "Durchschnittliche Wörter pro Szene",
  "writingStatusBar.savedProseNote":
    "Nur gespeicherter Text. Archivierte Inhalte sind ausgeschlossen; der Durchschnitt schließt leere Szenen ein.",
  "writingStatusBar.wordsPerChapter": "Wörter pro Kapitel",
  "writingStatusBar.chapterColumn": "Kapitel",
  "writingStatusBar.wordsColumn": "Wörter",
  "writingStatusBar.noChapters": "Noch keine Kapitel.",
  "writingStatusBar.sceneWords.one": "Szene: {count} Wort",
  "writingStatusBar.sceneWords.other": "Szene: {count} Wörter",
  "writingStatusBar.chapterWords.one": "Kapitel: {count} Wort",
  "writingStatusBar.chapterWords.other": "Kapitel: {count} Wörter",
  "writingStatusBar.projectWords.one": "Projekt: {count} Wort",
  "writingStatusBar.projectWords.other": "Projekt: {count} Wörter",
  "writingStatusBar.sessionWords.one": "Sitzung: {count} Wort",
  "writingStatusBar.sessionWords.other": "Sitzung: {count} Wörter",
  "writingStatusBar.sessionTooltip":
    "Netto gespeicherte Wörter seit dem Öffnen der App oder dem Zurücksetzen der Sitzung.",

  // WritingProgress
  "writingProgress.projectWords.one": "{count} Projektwort",
  "writingProgress.projectWords.other": "{count} Projektwörter",
  "writingProgress.todayTooltip":
    "Netto hinzugefügte Wörter durch gespeicherte Bearbeitungen heute. Löschungen verringern diese Summe.",
  "writingProgress.todayWithGoal.one": "Heute: {today} / {goal} Wort",
  "writingProgress.todayWithGoal.other": "Heute: {today} / {goal} Wörter",
  "writingProgress.todayWithoutGoal.one": "Heute: {today} Wort · Ziel aus",
  "writingProgress.todayWithoutGoal.other": "Heute: {today} Wörter · Ziel aus",
  "writingProgress.dailyGoal": "Tägliches Schreibziel",
  "writingProgress.sessionWords.one": "Sitzung: {count} Wort",
  "writingProgress.sessionWords.other": "Sitzung: {count} Wörter",
  "writingProgress.sessionTooltip":
    "Netto gespeicherte Wörter in diesem Projekt seit dem Öffnen der App oder dem Zurücksetzen.",
  "writingProgress.resetSession": "Schreibsitzung zurücksetzen",
  "writingProgress.reset": "Zurücksetzen",
  "writingProgress.streak.one": "{count} Tag Schreibserie",
  "writingProgress.streak.other": "{count} Tage Schreibserie",
  "writingProgress.resetError": "Ausstehenden Text speichern, bevor zurückgesetzt wird: {error}",

  // NovelEditor
  "novelEditor.placeholder": "Text verfassen…",
  "novelEditor.sceneProse": "Szenentext",
  "novelEditor.beatProse": "Beat-Text",
  "novelEditor.saving": "Wird gespeichert…",
  "novelEditor.saveError": "Speichern nicht möglich — Ihr Text ist noch hier",

  // PageView
  "pageView.title": "Szenentext",
  "pageView.words.one": "{count} Wort",
  "pageView.words.other": "{count} Wörter",
  "pageView.lockedPlaceholder": "Szene ist gesperrt",
  "pageView.placeholder": "Szenentext hier verfassen…",

  // Previously
  "previously.toggle": "Zuvor",
  "previously.toolsLabel": "Szenenwerkzeuge",
  "previously.heading": "Zuvor",
  "previously.loadError": "Vorheriger Szenenkontext konnte nicht geladen werden.",
  "previously.retry": "Erneut versuchen",
};

export default catalog;
