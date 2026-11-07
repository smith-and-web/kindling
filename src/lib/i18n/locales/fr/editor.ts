import type { Catalog } from "../../types";

/** Volet de scène, liste des temps forts, éditeurs de texte et statistiques d'écriture. */
const catalog: Catalog = {
  // ScenePanel
  "scenePanel.revisions": "Historique des brouillons",
  "scenePanel.openingRevisions": "Ouverture de l'historique…",
  "scenePanel.revisionsUnsavedError":
    "Enregistrez ou récupérez le texte non enregistré avant d'ouvrir l'historique.",
  "scenePanel.viewGroupLabel": "Affichage",
  "scenePanel.beatsTab": "Temps forts",
  "scenePanel.beatsTabTooltip": "Cartes de temps forts",
  "scenePanel.pageTab": "Page",
  "scenePanel.pageTabTooltip": "Texte en pleine page",
  "scenePanel.sequence": "Séquence",
  "scenePanel.chapter": "Chapitre",
  "scenePanel.breadcrumb": "{kind} · {name}",
  "scenePanel.pageEstimate": "~{pages} pages",
  "scenePanel.locked": "Verrouillée",
  "scenePanel.sceneTypeLabel": "Type de scène",
  "scenePanel.sceneTypeNormal": "Normal",
  "scenePanel.sceneTypeNotes": "Notes",
  "scenePanel.sceneTypeTodo": "À faire",
  "scenePanel.sceneTypeUnused": "Non utilisée",
  "scenePanel.statusLabel": "État",
  "scenePanel.sceneStatusDraft": "Brouillon",
  "scenePanel.sceneStatusRevised": "Révisée",
  "scenePanel.sceneStatusFinal": "Final",
  "scenePanel.planningLabel": "Planification",
  "scenePanel.planningFixed": "Fixée",
  "scenePanel.planningFlexible": "Flexible",
  "scenePanel.planningUndefined": "Non définie",
  "scenePanel.planningTooltip":
    "Contrôle la structure de cette scène : Non définie → Flexible → Fixée",
  "scenePanel.aboutPlanning": "À propos de la planification",
  "scenePanel.planningHelp":
    "Contrôle la structure de cette scène : Non définie, Flexible ou Fixée.",
  "scenePanel.tagsLabel": "Étiquettes",
  "scenePanel.rollingOutline": "Plan évolutif",
  "scenePanel.dismiss": "Fermer",
  "scenePanel.guideLedeBefore": "Le menu « ",
  "scenePanel.guidePlanning": "Planification",
  "scenePanel.guideLedeAfter":
    " » ci-dessus contrôle la structure de cette scène. Utilisez-le pour développer votre histoire progressivement :",
  "scenePanel.guideUndefinedDesc":
    "Une ébauche — vous savez qu'elle existe mais ne l'avez pas planifiée.",
  "scenePanel.guideFlexibleDesc": "Vous avez l'idée générale — un synopsis et une direction.",
  "scenePanel.guideFixedDesc": "Structure complète avec temps forts, fiches et notes.",
  "scenePanel.undefinedSceneTitle": "Scène non définie",
  "scenePanel.undefinedSceneBody":
    "Il s'agit d'une ébauche — vous savez qu'elle existe mais ne l'avez pas encore planifiée.",
  "scenePanel.undefinedSceneHelp":
    "Ajoutez un synopsis ci-dessous pour saisir l'idée générale, puis faites-la évoluer quand vous êtes prêt à la développer.",
  "scenePanel.switchToFlexible": "Passer en Flexible",
  "scenePanel.goStraightToFixed": "Passer directement en Fixée",
  "scenePanel.flexibleSceneTitle": "Scène flexible",
  "scenePanel.flexibleSceneBody":
    "Vous avez une idée pour cette scène mais n'avez pas encore fixé la structure.",
  "scenePanel.flexibleSceneHelp":
    "Utilisez le synopsis pour saisir votre intention. Quand vous êtes prêt à la diviser en temps forts, passez en Fixée.",
  "scenePanel.defineBeats": "Définir les temps forts",
  "scenePanel.sceneLockedTitle": "Cette scène est verrouillée",
  "scenePanel.chapterLockedHelp":
    "Le chapitre parent est verrouillé. Déverrouillez le chapitre pour modifier cette scène.",
  "scenePanel.sceneLockedHelp":
    "Déverrouillez cette scène depuis la barre latérale pour apporter des modifications.",
  "scenePanel.synopsisTitle": "Synopsis",
  "scenePanel.editSynopsis": "Modifier le synopsis",
  "scenePanel.synopsisPlaceholder": "Rédigez un court synopsis pour cette scène…",
  "scenePanel.saving": "Enregistrement…",
  "scenePanel.synopsisEditingHelp":
    "Appuyez sur Échap pour fermer. Les modifications sont enregistrées automatiquement.",
  "scenePanel.addSynopsis": "Ajouter un synopsis",
  "scenePanel.sceneLockedShort": "La scène est verrouillée",
  "scenePanel.synopsisSaveFailed":
    "Synopsis non enregistré : {error}. Votre brouillon est conservé pour réessai.",
  "scenePanel.retrySynopsisSave": "Réessayer l'enregistrement du synopsis",
  "scenePanel.retrySaving": "Réessayer l'enregistrement",
  "scenePanel.unsavedChanges": "Modifications non enregistrées",
  "scenePanel.referencesTitle": "Fiches",
  "scenePanel.loading": "Chargement…",
  "scenePanel.noReferences":
    "Aucune fiche liée pour l'instant. Liez-en depuis le volet des fiches.",
  "scenePanel.discoveryNotesTitle": "Notes de découverte",
  "scenePanel.hide": "Masquer",
  "scenePanel.show": "Afficher",
  "scenePanel.newNoteLabel": "Nouvelle note de découverte",
  "scenePanel.newNotePlaceholder": "Qu'avez-vous découvert ?",
  "scenePanel.addingNote": "Ajout…",
  "scenePanel.addNote": "Ajouter une note",
  "scenePanel.cancel": "Annuler",
  "scenePanel.editNoteLabel": "Modifier la note de découverte",
  "scenePanel.save": "Enregistrer",
  "scenePanel.edit": "Modifier",
  "scenePanel.promoting": "Promotion…",
  "scenePanel.promoteToBeat": "Promouvoir en temps fort",
  "scenePanel.delete": "Supprimer",
  "scenePanel.noNotes":
    "Aucune note de découverte pour l'instant. Capturez ce que vous trouvez pendant la rédaction — les notes ne font pas partie du manuscrit.",
  "scenePanel.contentTitle": "Contenu",
  "scenePanel.referenceLoadError": "Échec du chargement des fiches de la scène",
  "scenePanel.metadataUpdateError": "Échec de la mise à jour des métadonnées de la scène",
  "scenePanel.planningUpdateError": "Échec de la mise à jour de l'état de planification",
  "scenePanel.renameSceneError": "Échec du renommage de la scène : {error}",
  "scenePanel.addNoteError": "Échec de l'ajout de la note de découverte : {error}",
  "scenePanel.saveNoteError": "Échec de l'enregistrement de la note de découverte : {error}",
  "scenePanel.deleteNoteError": "Échec de la suppression de la note de découverte : {error}",
  "scenePanel.promoteNoteError": "Échec de la transformation de la note en temps fort : {error}",
  "scenePanel.modeSwitchUnsavedError":
    "Enregistrez, récupérez ou annulez le texte non enregistré dans Rechercher et remplacer avant de changer de mode d'édition.",
  "scenePanel.modeSwitchError": "Échec du changement de mode d'édition : {error}",
  "scenePanel.sceneChangedError":
    "Le projet ou la scène sélectionné a changé. Rouvrez Rechercher et remplacer.",
  "scenePanel.emptyTitle": "Sélectionnez une scène pour commencer à écrire",
  "scenePanel.emptyBody": "Choisissez une scène dans la barre latérale pour afficher son contenu.",
  "scenePanel.switchToBeatTitle": "Passer en vue Temps forts",
  "scenePanel.switchToBeatMessage":
    "Les modifications apportées dans la vue Page seront synchronisées avec les temps forts correspondants. Continuer ?",
  "scenePanel.switchConfirm": "Changer",

  // BeatView
  "beatView.beatsTitle": "Temps forts",
  "beatView.addBeat": "Ajouter un temps fort",
  "beatView.dragToReorder": "Faire glisser pour réorganiser",
  "beatView.beatTitleLabel": "Titre du temps fort",
  "beatView.beatWords.one": "{count} mot",
  "beatView.beatWords.other": "{count} mots",
  "beatView.beatMenu": "Menu du temps fort",
  "beatView.sceneLocked": "La scène est verrouillée",
  "beatView.prosePlaceholder": "Rédigez votre texte pour ce temps fort…",
  "beatView.openBeatProse": "Ouvrir le texte du temps fort {number}",
  "beatView.emptyTitle": "Aucun temps fort pour l'instant",
  "beatView.emptyBody":
    "Les temps forts proviennent d'un plan importé, ou vous pouvez les ajouter ici un par un.",
  "beatView.addFirstBeat": "Ajoutez votre premier temps fort",
  "beatView.newBeatLabel": "Nouveau temps fort",
  "beatView.newBeatPlaceholder": "Décrivez ce qui se passe dans ce temps fort…",
  "beatView.newBeatHelp": "Appuyez sur Entrée pour créer, Échap pour annuler.",
  "beatView.cancel": "Annuler",
  "beatView.creating": "Création…",
  "beatView.createBeat": "Créer le temps fort",
  "beatView.deleteTitle": "Supprimer le temps fort",
  "beatView.deleteFirstMessage":
    "Voulez-vous vraiment supprimer ce temps fort ? C'est le premier temps fort, son texte sera définitivement supprimé.",
  "beatView.deleteMergeMessage":
    "Voulez-vous vraiment supprimer ce temps fort ? Tout texte sera fusionné avec le temps fort précédent.",
  "beatView.menuRename": "Renommer",
  "beatView.menuSplit": "Diviser à la position du curseur",
  "beatView.menuMerge": "Fusionner avec le suivant",
  "beatView.menuMoveUp": "Déplacer vers le haut",
  "beatView.menuMoveDown": "Déplacer vers le bas",
  "beatView.menuDelete": "Supprimer",
  "beatView.createError": "Échec de la création du temps fort : {error}",
  "beatView.renameError": "Échec du renommage du temps fort : {error}",
  "beatView.deleteError": "Échec de la suppression du temps fort : {error}",
  "beatView.splitError": "Échec de la division du temps fort : {error}",
  "beatView.mergeError": "Échec de la fusion des temps forts : {error}",
  "beatView.transformUnsavedError":
    "Enregistrez les temps forts concernés ou récupérez leurs brouillons non enregistrés dans Rechercher et remplacer.",
  "beatView.reorderError": "Échec de la réorganisation des temps forts : {error}",
  "beatView.moveUpAnnouncement":
    "Le temps fort « {title} » a été déplacé vers le haut, à la position {position} sur {total}.",
  "beatView.moveDownAnnouncement":
    "Le temps fort « {title} » a été déplacé vers le bas, à la position {position} sur {total}.",

  // ProseToolbar
  "proseToolbar.label": "Mise en forme du texte",
  "proseToolbar.bold": "Gras",
  "proseToolbar.italic": "Italique",
  "proseToolbar.underline": "Souligné",
  "proseToolbar.monospace": "Chasse fixe",
  "proseToolbar.alignLeft": "Aligner à gauche",
  "proseToolbar.alignCenter": "Centrer",
  "proseToolbar.alignRight": "Aligner à droite",
  "proseToolbar.alignJustify": "Justifier",
  "proseToolbar.blockquote": "Citation",
  "proseToolbar.indent": "Retrait",
  "proseToolbar.undo": "Annuler",
  "proseToolbar.redo": "Rétablir",

  // SluglineInput
  "sluglineInput.sceneHeading": "En-tête de scène",
  "sluglineInput.interiorOrExterior": "Intérieur ou extérieur",
  "sluglineInput.location": "Lieu",
  "sluglineInput.locationPlaceholder": "Lieu",
  "sluglineInput.timeOfDay": "Moment de la journée",

  // WritingStatusBar
  "writingStatusBar.statisticsTitle": "Statistiques d'écriture",
  "writingStatusBar.hideStatistics": "Masquer les statistiques",
  "writingStatusBar.totalWords": "Total de mots",
  "writingStatusBar.scenesWithProse": "Scènes avec texte",
  "writingStatusBar.emptyScenes": "Scènes vides",
  "writingStatusBar.averageWordsPerScene": "Moyenne de mots par scène",
  "writingStatusBar.savedProseNote":
    "Texte enregistré uniquement. Le contenu archivé est exclu ; la moyenne inclut les scènes vides.",
  "writingStatusBar.wordsPerChapter": "Mots par chapitre",
  "writingStatusBar.chapterColumn": "Chapitre",
  "writingStatusBar.wordsColumn": "Mots",
  "writingStatusBar.noChapters": "Aucun chapitre pour l'instant.",
  "writingStatusBar.sceneWords.one": "Scène : {count} mot",
  "writingStatusBar.sceneWords.other": "Scène : {count} mots",
  "writingStatusBar.chapterWords.one": "Chapitre : {count} mot",
  "writingStatusBar.chapterWords.other": "Chapitre : {count} mots",
  "writingStatusBar.projectWords.one": "Projet : {count} mot",
  "writingStatusBar.projectWords.other": "Projet : {count} mots",
  "writingStatusBar.sessionWords.one": "Session : {count} mot",
  "writingStatusBar.sessionWords.other": "Session : {count} mots",
  "writingStatusBar.sessionTooltip":
    "Mots nets enregistrés depuis l'ouverture de l'application ou la réinitialisation de la session.",

  // WritingProgress
  "writingProgress.projectWords.one": "{count} mot du projet",
  "writingProgress.projectWords.other": "{count} mots du projet",
  "writingProgress.todayTooltip":
    "Mots nets ajoutés via les modifications enregistrées aujourd'hui. Les suppressions réduisent ce total.",
  "writingProgress.todayWithGoal.one": "Aujourd'hui : {today} / {goal} mot",
  "writingProgress.todayWithGoal.other": "Aujourd'hui : {today} / {goal} mots",
  "writingProgress.todayWithoutGoal.one": "Aujourd'hui : {today} mot · objectif désactivé",
  "writingProgress.todayWithoutGoal.other": "Aujourd'hui : {today} mots · objectif désactivé",
  "writingProgress.dailyGoal": "Objectif d'écriture quotidien",
  "writingProgress.sessionWords.one": "Session : {count} mot",
  "writingProgress.sessionWords.other": "Session : {count} mots",
  "writingProgress.sessionTooltip":
    "Mots nets enregistrés dans ce projet depuis l'ouverture de l'application ou la réinitialisation.",
  "writingProgress.resetSession": "Réinitialiser la session d'écriture",
  "writingProgress.reset": "Réinitialiser",
  "writingProgress.streak.one": "{count} jour de suite d'écriture",
  "writingProgress.streak.other": "{count} jours de suite d'écriture",
  "writingProgress.resetError":
    "Enregistrez votre texte en attente avant de réinitialiser : {error}",

  // NovelEditor
  "novelEditor.placeholder": "Rédigez votre texte…",
  "novelEditor.sceneProse": "Texte de la scène",
  "novelEditor.beatProse": "Texte du temps fort",
  "novelEditor.saving": "Enregistrement…",
  "novelEditor.saveError": "Impossible d'enregistrer — votre texte est toujours là",

  // PageView
  "pageView.title": "Texte de la scène",
  "pageView.words.one": "{count} mot",
  "pageView.words.other": "{count} mots",
  "pageView.lockedPlaceholder": "La scène est verrouillée",
  "pageView.placeholder": "Rédigez le texte de votre scène ici…",

  // Previously
  "previously.toggle": "Précédemment",
  "previously.toolsLabel": "Outils de scène",
  "previously.heading": "Précédemment",
  "previously.loadError": "Impossible de charger le contexte de la scène précédente.",
  "previously.retry": "Réessayer",
};

export default catalog;
