import type { Catalog } from "../../types";

/** Settings dialog and its Appearance, Author, Keyboard and Project panes. */
const catalog: Catalog = {
  // --- Settings dialog: navigation and chrome ---
  "dialog.title": "Settings",
  "dialog.closeLabel": "Close settings",
  "dialog.areasAria": "Settings areas",
  "dialog.groupPreferences": "Preferences",
  "dialog.groupProjects": "Projects",
  "dialog.sectionManuscript": "Manuscript",
  "dialog.sectionReferenceLibrary": "Reference Library",
  "dialog.areaAppearance": "Appearance & Guidance",
  "dialog.areaAuthor": "Author & Contact",
  "dialog.areaKeyboard": "Keyboard Shortcuts",
  "dialog.areaDetails": "Project Details",
  "dialog.areaReferences": "Reference Types",
  "dialog.areaTags": "Tags",
  "dialog.areaFields": "Custom Fields",
  "dialog.discardSettings": "Discard unsaved settings changes?",
  "dialog.discardProjectSwitch": "Discard unsaved changes for this project before switching?",
  "dialog.keepEditing": "Keep editing",
  "dialog.discard": "Discard changes",
  "dialog.loadingProjects": "Loading projects…",
  "dialog.retryLoadProjects": "Retry loading projects",
  "dialog.noProjects": "No projects available.",
  "dialog.projectLabel": "Project",
  "dialog.projectHelp": "Settings for this project. Your open manuscript stays in place.",
  "dialog.appliesToAll": "Applies to all projects.",
  "dialog.noProjectsHelp": "No projects yet. Create or import a project to configure its settings.",
  "dialog.loadProjectsFailed": "Could not load projects: {error}",

  // --- Appearance ---
  "appearance.lede":
    "Choose how kindling looks and how much guidance it shows. Changes apply immediately to all projects.",
  "appearance.title": "Appearance",
  "appearance.themeAria": "Theme",
  "appearance.themeDark": "Dark",
  "appearance.themeLight": "Light",
  "appearance.themeSystem": "System",
  "appearance.systemHelp": "“System” follows your operating system’s appearance setting.",
  "appearance.guidance": "Guidance",
  "appearance.showTips": "Show guidance tips",
  "appearance.tipsHelp":
    "A tip appears the first time you visit the sidebar, scene panel and references. Each tip shows once.",
  "appearance.showTipsAgain": "Show tips again",
  "appearance.takeTour": "Take the tour",
  "appearance.turnOnTips": "Turn on guidance tips to show them again.",
  "appearance.tipsReset": "Tips will appear again as you visit each area.",

  // --- Author & contact ---
  "author.lede":
    "Your author and contact details apply to manuscript title pages across all projects.",
  "author.loading": "Loading author details…",
  "author.infoTitle": "Author information",
  "author.nameLabel": "Author Name",
  "author.namePlaceholder": "Your legal name",
  "author.nameHelp":
    "Used in contact info on title pages. Projects can override this with a pen name.",
  "author.contactTitle": "Contact information",
  "author.contactHelp":
    "Optional details for manuscript title pages. Use any format that works for your country.",
  "author.address1": "Address Line 1",
  "author.address1Placeholder": "Street address or PO Box",
  "author.address2": "Address Line 2",
  "author.address2Placeholder": "City, State/Province, Postal Code, Country",
  "author.phone": "Phone",
  "author.phonePlaceholder": "+1 (555) 123-4567",
  "author.email": "Email",
  "author.emailPlaceholder": "author@email.com",
  "author.saved": "Author details saved.",
  "author.save": "Save author details",
  "author.loadFailedTitle": "Author details couldn’t be loaded",
  "author.loadFailedHelp":
    "If the settings file is damaged, reset author details to restore saving and exports. A recovery copy of the original file will be kept.",
  "author.retry": "Retry loading author details",
  "author.reset": "Reset author details",

  // --- Keyboard shortcuts ---
  "keyboard.lede":
    "Choose a shortcut, then press a new key combination. Changes apply immediately to all projects. Escape cancels recording; Tab moves to the next control.",
  "keyboard.systemHelp":
    "Standard text editing and system controls (such as copy, paste, undo, Tab, and Escape) keep their usual keys.",
  "keyboard.retryLoad": "Retry loading shortcuts",
  "keyboard.filterLabel": "Filter commands",
  "keyboard.resetDefaults": "Reset all to defaults",
  "keyboard.loading": "Loading shortcuts…",
  "keyboard.shortcutFor": "Shortcut for {label}",
  "keyboard.pressKeys": "Press keys…",
  "keyboard.unassigned": "Unassigned",
  "keyboard.clearFor": "Clear shortcut for {label}",
  "keyboard.clear": "Clear",
  "keyboard.noMatches": "No matching commands.",
  "keyboard.saved": "Keyboard shortcuts saved.",
  "keyboard.saveFailed": "Could not save keyboard shortcuts: {error}",
  "keyboard.invalidKey":
    "Use Command on macOS or Ctrl on Windows/Linux, with a letter, number, punctuation key, or F1–F24.",

  // --- Project details ---
  "project.ledePrefix": "Changes here apply only to ",
  "project.ledeSuffix": ".",
  "project.tagChanges": "Tag changes save as you make them.",
  "project.fieldChanges":
    "Custom field changes save as you make them. Fields are listed for saved, enabled reference types.",
  "project.optional": "(optional)",
  "project.penName": "Pen Name",
  "project.penNamePlaceholder": "Leave blank to use your author name",
  "project.penNameHelp":
    "If provided, this will be used as the byline on title pages instead of your author name.",
  "project.genre": "Genre",
  "project.genrePlaceholder": "e.g., Literary Fiction, Science Fiction, Mystery",
  "project.genreHelp": "Genre will be displayed on manuscript title pages.",
  "project.description": "Project Description",
  "project.descriptionPlaceholder": "Short summary or notes about this project",
  "project.dailyGoal": "Daily writing goal",
  "project.loadDailyGoalFailed": "Could not load daily goal: {error}",
  "project.dailyGoalHelp":
    "Net words added per day in this project. Set to 0 to turn off the goal. Changes apply today; earlier streak days keep their original goals.",
  "project.wordTarget": "Word Target",
  "project.wordTargetPlaceholder": "e.g., 80000",
  "project.referencesHelp":
    "Choose which reference types appear in this project’s References panel. Disabling a type keeps its existing entries.",
  "project.enableRefTypes": "Enable reference types to configure custom fields.",
  "project.saved": "Project changes saved.",
  "project.save": "Save project changes",
  "project.wordTargetInvalid": "Word target must be a non-negative whole number",
  "project.dailyGoalInvalid": "Daily goal must be a whole number between 0 and 1,000,000",
  "project.disabledDraft":
    "Save or cancel the {label} custom field draft before disabling that reference type.",
};

export default catalog;
