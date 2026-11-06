import type { Catalog } from "../../types";

/** Settings dialog and its Appearance, Author, Keyboard and Project panes. */
const catalog: Catalog = {
  // --- Settings dialog: navigation and chrome ---
  "dialog.title": "Einstellungen",
  "dialog.closeLabel": "Einstellungen schließen",
  "dialog.areasAria": "Einstellungsbereiche",
  "dialog.groupPreferences": "Einstellungen",
  "dialog.groupProjects": "Projekte",
  "dialog.sectionManuscript": "Manuskript",
  "dialog.sectionReferenceLibrary": "Referenzbibliothek",
  "dialog.areaAppearance": "Darstellung und Hinweise",
  "dialog.areaAuthor": "Autor und Kontakt",
  "dialog.areaKeyboard": "Tastaturkürzel",
  "dialog.areaDetails": "Projektdetails",
  "dialog.areaReferences": "Referenztypen",
  "dialog.areaTags": "Tags",
  "dialog.areaFields": "Benutzerdefinierte Felder",
  "dialog.discardSettings": "Nicht gespeicherte Einstellungsänderungen verwerfen?",
  "dialog.discardProjectSwitch":
    "Nicht gespeicherte Änderungen für dieses Projekt vor dem Wechsel verwerfen?",
  "dialog.keepEditing": "Weiter bearbeiten",
  "dialog.discard": "Änderungen verwerfen",
  "dialog.loadingProjects": "Projekte werden geladen…",
  "dialog.retryLoadProjects": "Projekte erneut laden",
  "dialog.noProjects": "Keine Projekte verfügbar.",
  "dialog.projectLabel": "Projekt",
  "dialog.projectHelp":
    "Einstellungen für dieses Projekt. Ihr geöffnetes Manuskript bleibt an Ort und Stelle.",
  "dialog.appliesToAll": "Gilt für alle Projekte.",
  "dialog.noProjectsHelp":
    "Noch keine Projekte. Projekt erstellen oder importieren, um Einstellungen zu konfigurieren.",
  "dialog.loadProjectsFailed": "Projekte konnten nicht geladen werden: {error}",

  // --- Appearance ---
  "appearance.lede":
    "Aussehen von kindling und Menge der angezeigten Hinweise wählen. Änderungen gelten sofort für alle Projekte.",
  "appearance.title": "Darstellung",
  "appearance.themeAria": "Thema",
  "appearance.themeDark": "Dunkel",
  "appearance.themeLight": "Hell",
  "appearance.themeSystem": "System",
  "appearance.systemHelp": "„System“ folgt der Darstellungseinstellung Ihres Betriebssystems.",
  "appearance.guidance": "Hinweise",
  "appearance.showTips": "Hinweistipps anzeigen",
  "appearance.tipsHelp":
    "Ein Tipp erscheint beim ersten Besuch der Seitenleiste, des Szenenbereichs und der Referenzen. Jeder Tipp wird einmal angezeigt.",
  "appearance.showTipsAgain": "Tipps wieder anzeigen",
  "appearance.takeTour": "Tour starten",
  "appearance.turnOnTips": "Hinweistipps aktivieren, um sie wieder anzuzeigen.",
  "appearance.tipsReset": "Tipps erscheinen wieder, wenn Sie jeden Bereich besuchen.",

  // --- Author & contact ---
  "author.lede":
    "Ihre Autoren- und Kontaktdaten gelten für Manuskript-Titelseiten in allen Projekten.",
  "author.loading": "Autorendetails werden geladen…",
  "author.infoTitle": "Autorenangaben",
  "author.nameLabel": "Autorenname",
  "author.namePlaceholder": "Ihr bürgerlicher Name",
  "author.nameHelp":
    "Wird in Kontaktinformationen auf Titelseiten verwendet. Projekte können dies mit einem Pseudonym überschreiben.",
  "author.contactTitle": "Kontaktangaben",
  "author.contactHelp":
    "Optionale Details für Manuskript-Titelseiten. Jedes Format verwenden, das für Ihr Land passt.",
  "author.address1": "Adresszeile 1",
  "author.address1Placeholder": "Straße oder Postfach",
  "author.address2": "Adresszeile 2",
  "author.address2Placeholder": "Ort, Bundesland, Postleitzahl, Land",
  "author.phone": "Telefon",
  "author.phonePlaceholder": "+49 (0) 30 1234567",
  "author.email": "E-Mail",
  "author.emailPlaceholder": "autor@email.de",
  "author.saved": "Autorendetails gespeichert.",
  "author.save": "Autorendetails speichern",
  "author.loadFailedTitle": "Autorendetails konnten nicht geladen werden",
  "author.loadFailedHelp":
    "Wenn die Einstellungsdatei beschädigt ist, Autorendetails zurücksetzen, um Speichern und Exporte wiederherzustellen. Eine Wiederherstellungskopie der Originaldatei wird behalten.",
  "author.retry": "Autorendetails erneut laden",
  "author.reset": "Autorendetails zurücksetzen",

  // --- Keyboard shortcuts ---
  "keyboard.lede":
    "Kürzel wählen, dann neue Tastenkombination drücken. Änderungen gelten sofort für alle Projekte. Escape bricht die Aufnahme ab; Tab wechselt zum nächsten Steuerelement.",
  "keyboard.systemHelp":
    "Standard-Textbearbeitung und Systemsteuerungen (wie Kopieren, Einfügen, Rückgängig, Tab und Escape) behalten ihre üblichen Tasten.",
  "keyboard.retryLoad": "Kürzel erneut laden",
  "keyboard.filterLabel": "Befehle filtern",
  "keyboard.resetDefaults": "Alle auf Standard zurücksetzen",
  "keyboard.loading": "Kürzel werden geladen…",
  "keyboard.shortcutFor": "Kürzel für {label}",
  "keyboard.pressKeys": "Tasten drücken…",
  "keyboard.unassigned": "Nicht zugewiesen",
  "keyboard.clearFor": "Kürzel für {label} löschen",
  "keyboard.clear": "Löschen",
  "keyboard.noMatches": "Keine passenden Befehle.",
  "keyboard.saved": "Tastaturkürzel gespeichert.",
  "keyboard.saveFailed": "Tastaturkürzel konnten nicht gespeichert werden: {error}",
  "keyboard.invalidKey":
    "Cmd auf macOS oder Strg unter Windows/Linux verwenden, mit einem Buchstaben, einer Zahl, einem Satzzeichen oder F1–F24.",

  // --- Project details ---
  "project.ledePrefix": "Änderungen hier gelten nur für ",
  "project.ledeSuffix": ".",
  "project.tagChanges": "Tag-Änderungen werden beim Vornehmen gespeichert.",
  "project.fieldChanges":
    "Änderungen an benutzerdefinierten Feldern werden beim Vornehmen gespeichert. Felder werden für gespeicherte, aktivierte Referenztypen aufgelistet.",
  "project.optional": "(optional)",
  "project.penName": "Pseudonym",
  "project.penNamePlaceholder": "Leer lassen, um Ihren Autorennamen zu verwenden",
  "project.penNameHelp":
    "Falls angegeben, wird dies als Byline auf Titelseiten statt Ihres Autorennamens verwendet.",
  "project.genre": "Genre",
  "project.genrePlaceholder": "z. B. Literaturfiktion, Science-Fiction, Krimi",
  "project.genreHelp": "Genre wird auf Manuskript-Titelseiten angezeigt.",
  "project.description": "Projektbeschreibung",
  "project.descriptionPlaceholder": "Kurze Zusammenfassung oder Notizen zu diesem Projekt",
  "project.dailyGoal": "Tägliches Schreibziel",
  "project.loadDailyGoalFailed": "Tagesziel konnte nicht geladen werden: {error}",
  "project.dailyGoalHelp":
    "Netto hinzugefügte Wörter pro Tag in diesem Projekt. Auf 0 setzen, um das Ziel zu deaktivieren. Änderungen gelten ab heute; frühere Streaktage behalten ihre ursprünglichen Ziele.",
  "project.wordTarget": "Wortziel",
  "project.wordTargetPlaceholder": "z. B. 80000",
  "project.referencesHelp":
    "Wählen, welche Referenztypen im Referenzbereich dieses Projekts angezeigt werden. Das Deaktivieren eines Typs behält seine vorhandenen Einträge.",
  "project.enableRefTypes":
    "Referenztypen aktivieren, um benutzerdefinierte Felder zu konfigurieren.",
  "project.saved": "Projektänderungen gespeichert.",
  "project.save": "Projektänderungen speichern",
  "project.wordTargetInvalid": "Wortziel muss eine nicht-negative ganze Zahl sein",
  "project.dailyGoalInvalid": "Tagesziel muss eine ganze Zahl zwischen 0 und 1.000.000 sein",
  "project.disabledDraft":
    "Den {label}-Entwurf für benutzerdefinierte Felder speichern oder abbrechen, bevor dieser Referenztyp deaktiviert wird.",
};

export default catalog;
