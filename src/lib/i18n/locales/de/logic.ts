import type { Catalog } from "../../types";

/** Copy produced outside components: stores, utils, import formats and command labels. */
const catalog: Catalog = {
  // --- UI store ---
  "ui.importing": "Importieren",
  "ui.startingImport": "Import wird gestartet…",
  "ui.importComplete": "Import abgeschlossen!",

  // --- Shortcut store and binding checks ---
  "shortcuts.loadFailed": "Tastaturkürzel konnten nicht geladen werden: {error}",
  "shortcuts.updateFailed": "Native Tastaturkürzel konnten nicht aktualisiert werden: {error}",
  "shortcuts.reserved":
    "Dieses Kürzel ist für Standard-Textbearbeitung oder Fenstersteuerung reserviert.",
  "shortcuts.conflict":
    "Wird bereits von {label} verwendet. Dieses Kürzel zuerst leeren oder ändern.",

  // --- Import helpers ---
  "import.selectScrivenerFolder": "Scrivener-Projektordner auswählen (.scriv)",
  "import.scrivenerFolderOnly": "Bitte einen Ordner auswählen, dessen Name auf .scriv endet",
  "import.unknownType": "Unbekannter Importtyp: {type}",
  "import.failed": "Import fehlgeschlagen: {error}",

  // --- Import formats ---
  "importFormat.plottr": "Plottr-Datei",
  "importFormat.markdown": "Markdown-Datei",
  "importFormat.ywriter": "yWriter-Datei",
  "importFormat.longform": "Longform-Index",
  "importFormat.longformVault": "Longform-Vault",
  "importFormat.novelwriter": "novelWriter-Projekt",
  "importFormat.scrivener": "Scrivener-Projekt",

  // --- Reference types ---
  "referenceType.characters": "Figuren",
  "referenceType.locations": "Orte",
  "referenceType.items": "Gegenstände",
  "referenceType.objectives": "Ziele",
  "referenceType.organizations": "Organisationen",
  "referenceType.timelines": "Zeitlinien",
  "referenceType.custom": "Notizen",

  // --- Editorial review validation ---
  "editorial.unknownSource": "Das Lektorat enthält eine unbekannte Manuskriptquelle.",
  "editorial.paragraphFormatting": "Das Lektorat enthält nicht unterstützte Absatzformatierung.",
  "editorial.unsupportedLink": "Das Lektorat enthält einen nicht unterstützten Link.",
  "editorial.outsidePassage":
    "Eine Rückmeldungsanmerkung verweist auf eine Passage außerhalb dieses Manuskripts.",
  "editorial.mismatchedPassage":
    "Eine Rückmeldungsanmerkung stimmt nicht mit ihrer ursprünglichen Passage überein.",
  "editorial.invalidAnchor": "Ein Kommentar hat einen ungültigen Passagenanker.",
  "editorial.noSources": "Das Lektorat hat keine Manuskriptquellen.",
  "editorial.reorderSources":
    "Diese Bearbeitung würde Manuskriptquellen neu anordnen. Stattdessen die Passage als Text kopieren.",
  "editorial.lostSource":
    "Ein Manuskriptabsatz hat seine Quelle verloren. Das Lektorat vor dem Fortfahren erneut öffnen.",
  "editorial.noCurrentPassages":
    "In diesem Lektorat gibt es keine aktuellen Manuskriptpassagen. Eine Szene wiederherstellen, bevor Rückmeldungen angewendet werden.",
  "editorial.passageChanged":
    "Diese Passage wurde geändert. Original-, aktuelle und vorgeschlagene Texte vergleichen und dann auswählen, wohin die Änderung gehört.",
  "editorial.overlap": "Diese Vorschläge überschneiden sich. Einzeln prüfen und annehmen.",
  "editorial.selectPassage": "Eine Passage im Manuskript auswählen.",

  // --- Prose save queue ---
  "proseSaves.draftsChanged":
    "Nicht gespeicherte Entwürfe haben sich geändert. Laden erneut versuchen, bevor sie verworfen werden.",
  "proseSaves.flushFailed":
    "Der neueste Text konnte nicht gespeichert werden. {errors} Speichern erneut versuchen oder nicht gespeicherte Entwürfe in Suchen und Ersetzen prüfen.",
  "proseFlush.message":
    "Nicht gespeicherten Text vor {action} speichern oder wiederherstellen. Nicht gespeicherte Entwürfe in Suchen und Ersetzen prüfen.",
  "proseFlush.action.exporting": "dem Exportieren",
  "proseFlush.action.syncing": "dem Synchronisieren",

  // --- Editorial recovery and re-anchoring ---
  "editorialSaves.recoveredMismatch":
    "Das wiederhergestellte Lektorat hat ein anderes Manuskript. Dieses Fenster offen lassen und eine Wiederherstellungskopie exportieren.",
  "localEditorial.reanchor":
    "Eine Passage im ursprünglichen Text dieser Szene auswählen, um ihre gespeicherte Anmerkung neu zu verankern.",

  // --- Review suggestions ---
  "revisions.suggestionClosed": "Vorschlag ist nicht mehr offen",
  "revisions.proseChanged":
    "Text geändert. Veraltete Vorschläge neu verankern, bevor sie angenommen werden.",
  "revisions.overlap": "Vorschläge überschneiden sich. Einzeln prüfen und annehmen.",

  // --- Utility errors ---
  "referenceCopy.failed": "Referenzen konnten nicht kopiert werden. Bitte erneut versuchen.",
  "synopsisSaves.changed":
    "Zusammenfassungsentwürfe haben sich geändert. Neueste Änderungen prüfen, bevor sie verworfen werden.",
  "exportPrototype.nameRequired": "Diesem Profil vor dem Speichern einen Namen geben.",
  "exportPrototype.startNumberInteger": "Kapitelstartnummer muss eine ganze Zahl sein.",
  "exportPrototype.unsupportedFormat":
    "Gespeicherte Prototypprofile haben ein nicht unterstütztes Format.",
  "exportPrototype.invalidProfile": "Ein gespeichertes Prototypprofil ist ungültig.",
  "exportPrototype.unsupportedOption": "Eine gespeicherte Prototypoption wird nicht unterstützt.",
  "exportPrototype.duplicateIds": "Gespeicherte Prototypprofile haben doppelte IDs.",
  "exportPrototype.fontSize": "Schriftgröße",
  "exportPrototype.lineSpacing": "Zeilenabstand",
  "exportPrototype.paragraphSpacing": "Absatzabstand",
  "exportPrototype.indent": "Erstzeilen-Einzug",
  "exportPrototype.margin": "Ränder",
  "exportPrototype.startNumber": "Kapitelstartnummer",
  "exportPrototype.rangeError": "{label} muss zwischen {min} und {max} liegen.",

  // --- Command categories ---
  "commandCategory.File": "Datei",
  "commandCategory.View": "Ansicht",
  "commandCategory.Edit": "Bearbeiten",
  "commandCategory.Help": "Hilfe",
  "commandCategory.Project": "Projekt",
  "commandCategory.Editor": "Editor",

  // --- Command labels (mirror shortcutDefinitions.json) ---
  "command.editorial_open.label": "Lektorats- oder Rückmeldungsdatei öffnen",
  "command.editorial_project.label": "Lektorat und Pakete",
  "command.find.label": "In Szene suchen",
  "command.find_replace.label": "Suchen und ersetzen",
  "command.find_project.label": "Im Projekt suchen und ersetzen",
  "command.export.label": "Projekt exportieren",
  "command.close_project.label": "Projekt schließen",
  "command.quit.label": "kindling beenden",
  "command.import_plottr.label": "Plottr importieren (.pltr)",
  "command.import_markdown.label": "Markdown importieren (.md)",
  "command.import_longform.label": "Longform importieren",
  "command.import_ywriter.label": "yWriter 7 importieren (.yw7)",
  "command.import_scrivener.label": "Scrivener 3 importieren (.scriv)",
  "command.import_novelwriter.label": "novelWriter importieren",
  "command.sync.label": "Aus Quelle synchronisieren",
  "command.toggle_sidebar.label": "Seitenleiste ein-/ausblenden",
  "command.toggle_references.label": "Referenzbereich ein-/ausblenden",
  "command.toggle_discovery_notes.label": "Entdeckungsnotizen ein-/ausblenden",
  "command.toggle_editor_mode.label": "Beat-/Seitenansicht umschalten",
  "command.detect_references.label": "Referenzen in Szene erkennen",
  "command.detect_all_references.label": "Referenzen in allen Szenen erkennen",
  "command.about.label": "Über kindling",
  "command.quick_start.label": "Schnellstartanleitung",
  "command.settings.label": "Einstellungen",
  "command.new_project.label": "Neues Projekt",
  "command.command_palette.label": "Befehlspalette",
  "command.send_feedback.label": "Rückmeldung senden",
  "command.editorial_comment.label": "Lektoratskommentar hinzufügen",
  "command.bold.label": "Fett",
  "command.italic.label": "Kursiv",
  "command.underline.label": "Unterstrichen",
  "command.strike.label": "Durchgestrichen",
  "command.code.label": "Nichtproportional",
  "command.blockquote.label": "Blockzitat",
  "command.align_left.label": "Linksbündig",
  "command.align_center.label": "Zentriert",
  "command.align_right.label": "Rechtsbündig",
  "command.align_justify.label": "Blocksatz",
};

export default catalog;
