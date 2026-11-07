import type { Catalog } from "../../types";

/** Texte produit en dehors des composants : stores, utilitaires, formats d'import et libellés de commandes. */
const catalog: Catalog = {
  // --- Store UI ---
  "ui.importing": "Importation",
  "ui.startingImport": "Démarrage de l'import…",
  "ui.importComplete": "Import terminé !",

  // --- Store de raccourcis et vérifications de liaisons ---
  "shortcuts.loadFailed": "Impossible de charger les raccourcis clavier : {error}",
  "shortcuts.updateFailed": "Impossible de mettre à jour les raccourcis clavier natifs : {error}",
  "shortcuts.reserved":
    "Ce raccourci est réservé à l'édition de texte standard ou aux contrôles de fenêtre.",
  "shortcuts.conflict": "Déjà utilisé par {label}. Effacez ou modifiez d'abord ce raccourci.",

  // --- Aides à l'import ---
  "import.selectScrivenerFolder": "Sélectionner le dossier de projet Scrivener (.scriv)",
  "import.scrivenerFolderOnly":
    "Veuillez sélectionner un dossier dont le nom se termine par .scriv",
  "import.unknownType": "Type d'import inconnu : {type}",
  "import.failed": "Échec de l'import : {error}",

  // --- Formats d'import ---
  "importFormat.plottr": "Fichier Plottr",
  "importFormat.markdown": "Fichier Markdown",
  "importFormat.ywriter": "Fichier yWriter",
  "importFormat.longform": "Index Longform",
  "importFormat.longformVault": "Coffre Longform",
  "importFormat.novelwriter": "Projet novelWriter",
  "importFormat.scrivener": "Projet Scrivener",

  // --- Types de fiches ---
  "referenceType.characters": "Personnages",
  "referenceType.locations": "Lieux",
  "referenceType.items": "Objets",
  "referenceType.objectives": "Objectifs",
  "referenceType.organizations": "Organisations",
  "referenceType.timelines": "Chronologies",
  "referenceType.custom": "Notes",

  // --- Validation de la relecture éditoriale ---
  "editorial.unknownSource": "La relecture contient une source de manuscrit inconnue.",
  "editorial.paragraphFormatting":
    "La relecture contient une mise en forme de paragraphe non prise en charge.",
  "editorial.unsupportedLink": "La relecture contient un lien non pris en charge.",
  "editorial.outsidePassage":
    "Une annotation de retour fait référence à un passage en dehors de ce manuscrit.",
  "editorial.mismatchedPassage":
    "Une annotation de retour ne correspond pas à son passage d'origine.",
  "editorial.invalidAnchor": "Un commentaire a une ancre de passage invalide.",
  "editorial.noSources": "La relecture n'a aucune source de manuscrit.",
  "editorial.reorderSources":
    "Cette modification réordonnerait les sources du manuscrit. Copiez le passage comme texte à la place.",
  "editorial.lostSource":
    "Un paragraphe du manuscrit a perdu sa source. Rouvrez la relecture avant de continuer.",
  "editorial.noCurrentPassages":
    "Il n'y a aucun passage de manuscrit actuel dans cette relecture. Restaurez une scène avant d'appliquer les retours.",
  "editorial.passageChanged":
    "Ce passage a changé. Comparez le texte original, actuel et suggéré, puis sélectionnez l'endroit où la modification appartient.",
  "editorial.overlap":
    "Ces suggestions se chevauchent. Examinez-les et acceptez-les individuellement.",
  "editorial.selectPassage": "Sélectionnez un passage dans le manuscrit.",

  // --- File d'enregistrement du texte ---
  "proseSaves.draftsChanged":
    "Les brouillons non enregistrés ont changé. Réessayez le chargement avant de les supprimer.",
  "proseSaves.flushFailed":
    "Impossible d'enregistrer le dernier texte. {errors} Réessayez l'enregistrement ou examinez les brouillons non enregistrés dans Rechercher et remplacer.",
  "proseFlush.message":
    "Enregistrez ou récupérez le texte non enregistré avant {action}. Examinez les brouillons non enregistrés dans Rechercher et remplacer.",
  "proseFlush.action.exporting": "l'export",
  "proseFlush.action.syncing": "la synchronisation",

  // --- Récupération éditoriale et ré-ancrage ---
  "editorialSaves.recoveredMismatch":
    "La relecture récupérée a un manuscrit différent. Gardez cette fenêtre ouverte et exportez une copie de récupération.",
  "localEditorial.reanchor":
    "Sélectionnez un passage dans le texte d'origine de cette scène pour ré-ancrer son annotation enregistrée.",

  // --- Suggestions de relecture ---
  "revisions.suggestionClosed": "La suggestion n'est plus ouverte",
  "revisions.proseChanged":
    "Le texte a changé. Ré-ancrez les suggestions obsolètes avant de les accepter.",
  "revisions.overlap":
    "Les suggestions se chevauchent. Examinez-les et acceptez-les individuellement.",

  // --- Erreurs utilitaires ---
  "referenceCopy.failed": "Impossible de copier les fiches. Veuillez réessayer.",
  "synopsisSaves.changed":
    "Les brouillons de synopsis ont changé. Examinez les dernières modifications avant de les supprimer.",
  "exportPrototype.nameRequired": "Donnez un nom à ce profil avant de l'enregistrer.",
  "exportPrototype.startNumberInteger":
    "Le numéro de départ du chapitre doit être un nombre entier.",
  "exportPrototype.unsupportedFormat":
    "Les profils de prototype enregistrés ont un format non pris en charge.",
  "exportPrototype.invalidProfile": "Un profil de prototype enregistré est invalide.",
  "exportPrototype.unsupportedOption":
    "Une option de prototype enregistrée n'est pas prise en charge.",
  "exportPrototype.duplicateIds":
    "Les profils de prototype enregistrés ont des identifiants en double.",
  "exportPrototype.fontSize": "Taille de police",
  "exportPrototype.lineSpacing": "Interligne",
  "exportPrototype.paragraphSpacing": "Espacement des paragraphes",
  "exportPrototype.indent": "Retrait de première ligne",
  "exportPrototype.margin": "Marges",
  "exportPrototype.startNumber": "Numéro de départ du chapitre",
  "exportPrototype.rangeError": "{label} doit être compris entre {min} et {max}.",

  // --- Catégories de commandes ---
  "commandCategory.File": "Fichier",
  "commandCategory.View": "Affichage",
  "commandCategory.Edit": "Édition",
  "commandCategory.Help": "Aide",
  "commandCategory.Project": "Projet",
  "commandCategory.Editor": "Éditeur",

  // --- Libellés de commandes (miroir de shortcutDefinitions.json) ---
  "command.editorial_open.label": "Ouvrir une relecture ou des retours",
  "command.editorial_project.label": "Relecture éditoriale et paquets",
  "command.find.label": "Rechercher dans la scène",
  "command.find_replace.label": "Rechercher et remplacer",
  "command.find_project.label": "Rechercher et remplacer dans le projet",
  "command.export.label": "Exporter le projet",
  "command.close_project.label": "Fermer le projet",
  "command.quit.label": "Quitter kindling",
  "command.import_plottr.label": "Importer Plottr (.pltr)",
  "command.import_markdown.label": "Importer Markdown (.md)",
  "command.import_longform.label": "Importer Longform",
  "command.import_ywriter.label": "Importer yWriter 7 (.yw7)",
  "command.import_scrivener.label": "Importer Scrivener 3 (.scriv)",
  "command.import_novelwriter.label": "Importer novelWriter",
  "command.sync.label": "Synchroniser depuis la source",
  "command.toggle_sidebar.label": "Afficher/masquer la barre latérale",
  "command.toggle_references.label": "Afficher/masquer le volet des fiches",
  "command.toggle_discovery_notes.label": "Afficher/masquer les notes de découverte",
  "command.toggle_editor_mode.label": "Basculer entre la vue Temps forts et Page",
  "command.detect_references.label": "Détecter les fiches dans la scène",
  "command.detect_all_references.label": "Détecter les fiches dans toutes les scènes",
  "command.about.label": "À propos de kindling",
  "command.quick_start.label": "Guide de démarrage rapide",
  "command.settings.label": "Réglages",
  "command.new_project.label": "Nouveau projet",
  "command.command_palette.label": "Palette de commandes",
  "command.send_feedback.label": "Envoyer des retours",
  "command.editorial_comment.label": "Ajouter un commentaire éditorial",
  "command.bold.label": "Gras",
  "command.italic.label": "Italique",
  "command.underline.label": "Souligné",
  "command.strike.label": "Barré",
  "command.code.label": "Chasse fixe",
  "command.blockquote.label": "Citation",
  "command.align_left.label": "Aligner à gauche",
  "command.align_center.label": "Centrer",
  "command.align_right.label": "Aligner à droite",
  "command.align_justify.label": "Justifier",
};

export default catalog;
