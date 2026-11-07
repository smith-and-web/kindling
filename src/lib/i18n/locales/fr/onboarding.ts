import type { Catalog } from "../../types";

/** Écran de démarrage, intégration, démarrage rapide, import et boîtes de dialogue À propos/Retours. */
const catalog: Catalog = {
  // StartScreen
  "startScreen.quickStart": "Démarrage rapide",
  "startScreen.tagline": "Lancez votre brouillon",
  "startScreen.startGroupLabel": "Démarrer",
  "startScreen.newProject": "Nouveau projet",
  "startScreen.startFromScratch": "Partir de zéro",
  "startScreen.openingSample": "Ouverture de l'exemple…",
  "startScreen.sampleProject": "Projet exemple",
  "startScreen.exploreKindling": "Explorer kindling d'abord",
  "startScreen.openReviewPackage": "Ouvrir un paquet de relecture",
  "startScreen.openReviewPackageHint": "Ouvrir un fichier de relecture ou de retours",
  "startScreen.allProjects": "Tous les projets",
  "startScreen.recentProjects": "Projets récents",
  "startScreen.showRecent": "Afficher les récents",
  "startScreen.viewAll": "Voir tout",
  "startScreen.openFailedTitle": "Impossible d'ouvrir « {name} »",
  "startScreen.opening": "Ouverture…",
  "startScreen.modifiedLabel": "Modifié le {date}",
  "startScreen.deleteNamed": "Supprimer {name}",
  "startScreen.deleteProject": "Supprimer le projet",
  "startScreen.projectsLabel": "Projets",
  "startScreen.emptyTitle": "Vos projets apparaîtront ici",
  "startScreen.emptyMessage":
    "Démarrez un nouveau projet, essayez l'exemple, ou importez un plan que vous avez déjà.",
  "startScreen.importOutline": "Importer un plan",
  "startScreen.importHelp":
    "Les chapitres, scènes et temps forts de votre plan deviennent des invites dans la vue d'écriture.",
  "startScreen.importIndexOrVault": "Index ou coffre",
  "startScreen.importProjectFolder": "Dossier de projet",
  "startScreen.blankSource": "Vierge",
  "startScreen.deleteTitle": "Supprimer « {name} » ?",
  "startScreen.deleteMessage":
    "Cela supprime définitivement le projet et tous ses chapitres, scènes, temps forts et instantanés. Cette action est irréversible.",
  "startScreen.importOpeningTitle": "Ouverture du projet exemple",
  "startScreen.importOpeningStatus": "Création du projet exemple…",
  "startScreen.errorCreateSample": "Échec de la création du projet exemple : {error}",
  "startScreen.errorLoadProjects": "Échec du chargement des projets : {error}",
  "startScreen.errorDeleteProject": "Échec de la suppression du projet : {error}",

  // Intégration
  welcomeTitle: "Bienvenue dans kindling",
  tourSidebarTitle: "Chapitres et scènes",
  tourSidebarSubtitle: "La barre latérale gauche",
  tourEditorTitle: "Synopsis et temps forts",
  tourEditorSubtitle: "L'éditeur principal",
  tourReferencesTitle: "Fiches",
  tourReferencesSubtitle: "Le volet de droite",
  importTitle: "Importez votre plan",
  readyToImport: "Prêt à importer « {name} »",
  progressGettingStarted: "Premiers pas",
  progressImport: "Import",
  progressTour: "Visite",
  stepProgress: "Étape {current} sur {total}",
  progressAria: "{label}, étape {current} sur {total}",
  skipOnboarding: "Passer l'intégration",
  welcomeLede:
    "Transformez votre plan en brouillon achevé. Importez la structure de votre histoire et commencez à écrire scène par scène.",
  importHintIndexOrVault: "Index ou coffre",
  importHintProjectFolder: "Dossier de projet",
  importOpeningTitle: "Ouverture du projet exemple",
  importOpeningStatus: "Création du projet exemple…",
  errorCreateSample: "Échec de la création du projet exemple : {error}",
  errorImportFailed: "Échec de l'import : {error}",
  readingOutline: "Lecture de votre plan…",
  previewLede: "Ce plan contient les éléments suivants.",
  factChapters: "Chapitres",
  factScenes: "Scènes",
  factBeats: "Temps forts",
  factCharacters: "Personnages",
  factLocations: "Lieux",
  outlineUnreadable: "Impossible de lire ce plan",
  chooseFormatLede:
    "Choisissez le format de votre plan. kindling le prévisualise avant l'import.",
  trySample: "Essayer le projet exemple",
  startFromScratch: "Démarrer un nouveau projet de zéro",
  skipAndImport: "Passer et commencer l'import",
  takeTour: "Faire la visite",
  trySampleShort: "Essayer le projet exemple",
  chooseDifferentFile: "Choisir un autre fichier",
  importProject: "Importer le projet",
  tryDifferentFile: "Essayer un autre fichier",
  backToTour: "Retour à la visite",
  importLater: "J'importerai plus tard",
  skipTour: "Passer la visite",
  startImporting: "Commencer l'import",

  // Visite d'intégration : barre latérale
  mockChapterName: "La lettre",
  mockChapterBadge: "Chapitre",
  mockSceneOnCliff: "Sur la falaise",
  mockCurrentScene: "Scène actuelle",
  mockSceneSeventhStep: "La septième marche",
  mockSceneBadge: "Scène",
  sidebarPointClickChapter: "Cliquez sur un chapitre",
  sidebarPointClickChapterRest: " pour développer ou réduire ses scènes.",
  sidebarPointClickScene: "Cliquez sur une scène",
  sidebarPointClickSceneRest: " pour la charger dans l'éditeur.",
  sidebarPointHighlightPrefix: "La scène ",
  sidebarPointHighlight: "mise en surbrillance",
  sidebarPointHighlightSuffix: " est votre scène de travail actuelle.",
  sidebarPointPartsPrefix: "Les projets importés et les modèles peuvent inclure des ",
  sidebarPointParts: "Parties",
  sidebarPointPartsSuffix: " (actes) comme groupes réductibles au-dessus des chapitres.",
  sidebarPointScreenplayPrefix: "Les projets de scénario utilisent ",
  sidebarPointScreenplayStructure: "Actes → Séquences → Scènes",
  sidebarPointScreenplayMiddle: " au lieu de Chapitres → Scènes, avec des estimations du ",
  sidebarPointPageCount: "nombre de pages",
  sidebarPointScreenplaySuffix: " à côté de chaque acte.",

  // Visite d'intégration : éditeur
  mockSynopsis: "Synopsis",
  mockSceneSynopsisBadge: "Synopsis de scène",
  mockSynopsisProse:
    "Le héros reçoit l'appel à l'aventure et doit décider s'il laissera derrière lui son monde ordinaire…",
  mockBeatMessenger: "Le messager arrive",
  mockStoryBeatBadge: "Temps fort",
  mockBeatDecision: "La décision",
  editorPointSynopsisPrefix: "Le ",
  editorPointSynopsis: "synopsis",
  editorPointSynopsisSuffix: " vous donne l'aperçu de la scène depuis votre plan.",
  editorPointBeatsStrong: "Les temps forts",
  editorPointBeatsRest: " sont les moments clés de l'histoire au sein de chaque scène.",
  editorPointPrompts:
    "Utilisez les temps forts comme invites d'écriture pour rédiger chaque section.",
  editorPointSwitchPrefix: "Basculez entre ",
  editorPointSwitchBeats: "Temps forts",
  editorPointSwitchMiddle: " (guidé par le plan) et ",
  editorPointSwitchPage: "Page",
  editorPointSwitchSuffix: " (écriture libre) en haut de la scène.",
  editorPointSluglinesPrefix: "Dans les scénarios, les titres de scènes sont des ",
  editorPointSluglines: "en-têtes de scène",
  editorPointSluglinesMiddle: " (par exemple, ",
  editorPointSluglinesSuffix: "). kindling suggère des lieux depuis vos fiches.",

  // Visite d'intégration : fiches
  referencesTabCharacters: "Personnages",
  referencesTabLocations: "Lieux",
  referencesTabItems: "Objets",
  mockCharacterName: "Elena",
  mockCharacterRole: "Protagoniste",
  mockLinkedBadge: "Liée",
  mockMentorName: "Marcus",
  mockMentorRole: "Mentor",
  mockAdd: "Ajouter",
  mockCollapseAll: "Tout réduire",
  mockSortAZ: "Trier A–Z",
  mockHidePanel: "Masquer le volet",
  referencesPointCharacters: "Personnages",
  referencesPointCharactersMiddle:
    " montre qui apparaît dans la scène actuelle ; ",
  referencesPointLocations: "Lieux",
  referencesPointLocationsRest: " montre où elle se déroule.",
  referencesPointAddPrefix: "Utilisez ",
  referencesPointAdd: "Ajouter",
  referencesPointAddSuffix: " pour rechercher et lier des fiches à une scène.",
  referencesPointDetectPrefix: "kindling peut ",
  referencesPointDetect: "détecter",
  referencesPointDetectMiddle:
    " les mentions de personnages et de lieux dans votre texte. Cherchez le groupe ",
  referencesPointSuggested: "Suggérées",
  referencesPointSuggestedRest: ".",
  referencesPointHidePanel: "Masquer le volet",
  referencesPointHidePanelRest:
    " réduit la barre latérale pour vous permettre de vous concentrer sur l'écriture.",

  // QuickStartDialog
  "quickStartDialog.title": "Démarrage rapide",
  "quickStartDialog.importHeading": "Importez votre plan",
  "quickStartDialog.importLede":
    "kindling fonctionne avec votre plan existant. Importez depuis l'écran de démarrage ou Fichier → Importer.",
  "quickStartDialog.plottr": " — fichiers .pltr",
  "quickStartDialog.ywriter": " — fichiers .yw7",
  "quickStartDialog.markdown":
    " — Fichier .md unique avec structure # Chapitre, ## Scène, - Temps fort",
  "quickStartDialog.longform": " — Coffre ou fichier index Obsidian",
  "quickStartDialog.scrivener": " — Paquets de projet .scriv",
  "quickStartDialog.sidebarHeading": "Barre latérale — chapitres et scènes",
  "quickStartDialog.sidebarLede":
    "La barre latérale gauche montre la structure de votre projet. Cliquez sur un chapitre pour développer ou réduire ses scènes. Cliquez sur une scène pour la charger dans l'éditeur.",
  "quickStartDialog.sidebarChapters":
    "Les chapitres regroupent les scènes connexes",
  "quickStartDialog.sidebarScenes":
    "Les scènes sont vos unités d'écriture principales — chacune a un synopsis et des temps forts",
  "quickStartDialog.sidebarContextMenu":
    "Clic droit sur les chapitres ou scènes pour plus d'options (réorganiser, archiver, verrouiller)",
  "quickStartDialog.scenePanelHeading": "Volet de scène — synopsis et temps forts",
  "quickStartDialog.scenePanelLede":
    "Lorsque vous sélectionnez une scène, la zone principale affiche son synopsis et ses temps forts. Les temps forts sont les moments clés — développez chacun pour rédiger le texte.",
  "quickStartDialog.synopsisLabel": "Synopsis",
  "quickStartDialog.beatsLabel": "Temps forts",
  "quickStartDialog.charactersLabel": "Personnages",
  "quickStartDialog.locationsLabel": "Lieux",
  "quickStartDialog.synopsisBullet":
    " — Bref aperçu de la scène (depuis votre plan)",
  "quickStartDialog.beatsBullet":
    " — Moments clés ; cliquez pour développer et rédiger",
  "quickStartDialog.beatsContextMenu":
    "Clic droit sur les temps forts pour diviser, fusionner ou supprimer. Faites glisser pour réorganiser.",
  "quickStartDialog.discoveryNotesLabel": "Notes de découverte",
  "quickStartDialog.discoveryNotesRest":
    " — Capturez les idées en écrivant ; promouvez-les en temps forts quand elles sont prêtes",
  "quickStartDialog.referencesHeading": "Volet des fiches",
  "quickStartDialog.referencesLede":
    "Le volet droit affiche les personnages et lieux liés à la scène actuelle. Utilisez-le pour suivre qui apparaît où.",
  "quickStartDialog.charactersBullet": " — Qui apparaît dans cette scène",
  "quickStartDialog.locationsBullet": " — Où la scène se déroule",
  "quickStartDialog.linkReferences":
    "Liez des personnages et des lieux depuis votre plan, ou ajoutez-les dans kindling",
  "quickStartDialog.tipsHeading": "Conseils",
  "quickStartDialog.exportFallback": "Exporter",
  "quickStartDialog.exportRest": " — Exportez votre projet",
  "quickStartDialog.discoveryNotesToggleRest":
    " — Afficher/masquer les notes de découverte dans le volet de scène",
  "quickStartDialog.snapshotsLabel": "Instantanés",
  "quickStartDialog.snapshotsRest":
    " — Créez des points de version avant les grands changements (barre latérale)",
  "quickStartDialog.gotIt": "Compris",

  // GuidanceOverlay
  "guidanceOverlay.tipTitle": "Conseil",
  "guidanceOverlay.disableTips": "Désactiver les conseils",
  "guidanceOverlay.disableTipsTitle": "Ne plus afficher les conseils",
  "guidanceOverlay.gotIt": "Compris",
  "guidanceOverlay.sidebarMessage":
    "Votre plan est ici. Cliquez sur les chapitres pour développer, faites glisser pour réorganiser.",
  "guidanceOverlay.scenePanelMessage":
    "Modifiez les temps forts et les scènes ici. Capturez les idées dans les notes de découverte.",
  "guidanceOverlay.referencesMessage":
    "Liez des personnages, des lieux et des objets aux scènes. Utilisez le bouton + pour rechercher et ajouter des fiches, ou acceptez les fiches suggérées que kindling trouve dans votre texte.",
  "guidanceOverlay.syncMessage":
    "Synchroniser depuis la source réimporte votre fichier de plan quand vous avez apporté des modifications ailleurs. Utilisez-le pour récupérer les modifications de Plottr, yWriter, Longform, Scrivener ou Markdown.",
  "guidanceOverlay.planningStatusMessage":
    "Utilisez l'état de planification (Fixée, Flexible, Non définie) pour contrôler la structure de chaque scène. Commencez librement et affinez au fur et à mesure.",
  "guidanceOverlay.screenplayMessage":
    "Il s'agit d'un projet de scénario. Les actes et séquences remplacent les chapitres. Utilisez des en-têtes de scène pour les titres — l'estimation du nombre de pages se met à jour au fil de votre écriture.",

  // NewProjectDialog
  "newProjectDialog.title": "Nouveau projet",
  "newProjectDialog.projectTypeLegend": "Type de projet",
  "newProjectDialog.novel": "Roman",
  "newProjectDialog.screenplay": "Scénario",
  "newProjectDialog.nameLabel": "Nom",
  "newProjectDialog.namePlaceholder": "Saisissez le nom du projet…",
  "newProjectDialog.targetLengthLabel": "Longueur cible",
  "newProjectDialog.lengthShort": "Court (< 30 pages)",
  "newProjectDialog.lengthFeature": "Long métrage (90–120 pages)",
  "newProjectDialog.lengthLongFeature": "Long métrage étendu (120–180 pages)",
  "newProjectDialog.templateLegend": "Modèle de structure",
  "newProjectDialog.optional": "(facultatif)",
  "newProjectDialog.removeTemplate": "Supprimer le modèle",
  "newProjectDialog.browseTemplates": "Parcourir les modèles…",
  "newProjectDialog.creating": "Création…",
  "newProjectDialog.create": "Créer",
  "newProjectDialog.defaultName": "Mon projet",
  "newProjectDialog.nameEmpty": "Le nom ne peut pas être vide",
  "newProjectDialog.createFailed": "Échec de la création du projet",

  // ImportLongformDialog
  "importLongformDialog.title": "Importer un projet Longform",
  "importLongformDialog.lede":
    "Choisissez comment importer votre projet Longform. Utilisez le fichier index si vous avez déjà une note de projet Longform, ou sélectionnez un dossier de coffre pour détecter automatiquement le projet.",
  "importLongformDialog.indexTitle": "Choisir le fichier index Longform",
  "importLongformDialog.indexHint":
    "Sélectionnez la note index .md avec le frontmatter Longform",
  "importLongformDialog.vaultTitle": "Choisir le dossier du coffre Obsidian",
  "importLongformDialog.vaultHint": "Analyser le coffre pour trouver des projets Longform",

  // ImportProgressDialog
  "importProgressDialog.statusFallback": "Lecture de votre plan…",

  // AboutDialog
  "aboutDialog.title": "À propos de kindling",
  "aboutDialog.tagline":
    "Lancez votre brouillon — faites le pont entre le plan et le texte.",
  "aboutDialog.versionLabel": "Version",
  "aboutDialog.unknownVersion": "inconnue",
  "aboutDialog.sendFeedback": "Envoyer des retours",
  "aboutDialog.github": "Dépôt GitHub",
  "aboutDialog.reportIssue": "Signaler un problème",
  "aboutDialog.releaseNotes": "Notes de version",
  "aboutDialog.linkError": "Impossible d'ouvrir le lien : {error}",

  // FeedbackDialog
  "feedbackDialog.title": "Envoyer des retours",
  "feedbackDialog.successTitle": "Merci pour vos retours !",
  "feedbackDialog.successMessage": "Votre message a été envoyé à l'équipe kindling.",
  "feedbackDialog.kindsLegend": "Quel type de retours ?",
  "feedbackDialog.typeBug": "Bogue",
  "feedbackDialog.typeFeature": "Fonctionnalité",
  "feedbackDialog.typeRating": "Évaluation",
  "feedbackDialog.ratingLegend": "Comment évaluez-vous kindling ?",
  "feedbackDialog.star.one": "{count} étoile",
  "feedbackDialog.star.other": "{count} étoiles",
  "feedbackDialog.ratingOfFive": "{rating} sur 5",
  "feedbackDialog.summaryLabel": "Résumé",
  "feedbackDialog.optional": "(facultatif)",
  "feedbackDialog.summaryPlaceholder": "Un court titre",
  "feedbackDialog.messageLabel": "Message",
  "feedbackDialog.messagePlaceholder": "Dites-nous ce que vous pensez…",
  "feedbackDialog.disclosure":
    "L'envoi inclut ce que vous avez saisi ici ainsi que votre version de kindling, votre système d'exploitation et votre langue. Rien de votre manuscrit n'est joint.",
  "feedbackDialog.validationSummaryTooLong":
    "Le résumé doit comporter {count} caractères ou moins.",
  "feedbackDialog.validationRating": "Veuillez sélectionner une évaluation de 1 à 5.",
  "feedbackDialog.validationMessageRequired": "Veuillez saisir un message.",
  "feedbackDialog.validationMessageTooLong":
    "Le message doit comporter {count} caractères ou moins.",
  "feedbackDialog.errorTitle": "Impossible d'envoyer vos retours.",
  "feedbackDialog.formKept": "Votre formulaire est conservé tel quel.",
  "feedbackDialog.tryAgain": "Réessayer",
  "feedbackDialog.sending": "Envoi…",
  "feedbackDialog.send": "Envoyer les retours",

  // UpdateBanner
  "updateBanner.ready": "kindling v{version} est prêt — Redémarrez pour mettre à jour",
  "updateBanner.restarting": "Redémarrage…",
  "updateBanner.restart": "Redémarrer",
  "updateBanner.dismiss": "Fermer",

  // CommandPalette
  "commandPalette.dialogLabel": "Palette de commandes",
  "commandPalette.placeholder": "Tapez une commande ou recherchez…",
  "commandPalette.searchLabel": "Rechercher des commandes",
  "commandPalette.listLabel": "Commandes",
  "commandPalette.emptyTitle": "Aucune commande correspondante",
  "commandPalette.emptyMessage": "Vérifiez l'orthographe ou essayez un mot plus court.",
  "commandPalette.navigate": "pour naviguer",
  "commandPalette.run": "pour exécuter",
  "commandPalette.close": "pour fermer",
};

export default catalog;
