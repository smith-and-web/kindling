import type { Catalog } from "../../types";

/** Espace de relecture éditoriale, manuscrit, barre latérale de relecture, historique et instantanés. */
const catalog: Catalog = {
  // --- EditorialWorkspace : menu, état et erreurs ---
  "editorialWorkspace.menuDraftHistory": "Historique des brouillons",
  "editorialWorkspace.menuStartSuggestions": "Lancer les suggestions depuis le texte actuel",
  "editorialWorkspace.menuReviewPackages": "Paquets de relecture et tours…",
  "editorialWorkspace.menuExportFeedback": "Exporter les retours",
  "editorialWorkspace.exportRecoveryCopy": "Exporter une copie de récupération",
  "editorialWorkspace.openReviewFile": "Ouvrir une relecture ou des retours…",
  "editorialWorkspace.defaultWriterName": "Auteur",
  "editorialWorkspace.savedLocally": "Enregistré localement",
  "editorialWorkspace.notSaved": "Non enregistré — réessayez ou exportez une copie de récupération",
  "editorialWorkspace.saving": "Enregistrement…",
  "editorialWorkspace.recoveryJournalError":
    "Journal de récupération : {error}. Gardez cette fenêtre ouverte jusqu'à ce que l'enregistrement réussisse.",
  "editorialWorkspace.sceneLockedReview":
    "Déverrouillez cette scène avant de modifier sa relecture.",
  "editorialWorkspace.sceneLockedFeedback":
    "Déverrouillez cette scène avant d'ajouter des retours.",
  "editorialWorkspace.scenesLockedBulk":
    "Déverrouillez les scènes sélectionnées avant de statuer sur leurs suggestions.",
  "editorialWorkspace.selectOriginalScene": "Sélectionnez un passage dans la scène d'origine.",
  "editorialWorkspace.conversationUnavailable": "Cette conversation n'est plus disponible.",
  "editorialWorkspace.generalFeedback": "Retours généraux",
  "editorialWorkspace.thisScene": "cette scène",
  "editorialWorkspace.modeBeatLower": "temps fort",
  "editorialWorkspace.modePageLower": "page",
  "editorialWorkspace.modeBeat": "Temps forts",
  "editorialWorkspace.modePage": "Page",
  "editorialWorkspace.unavailableInactive":
    "Enregistré sur le texte inactif {mode} dans « {scene} ». Revenez à l'écriture et passez cette scène en mode {target} pour agir sur ces retours.",
  "editorialWorkspace.exportBlockedNoScenes": "Ce projet n'a encore aucune scène à inclure.",
  "editorialWorkspace.exportBlockedNoName": "Nommez le tour de relecture avant de l'exporter.",
  "editorialWorkspace.exportBlockedNoChapters": "Choisissez au moins un chapitre.",
  "editorialWorkspace.defaultRoundName": "Passe éditoriale",

  // --- EditorialWorkspace : en-tête et chrome du manuscrit ---
  "editorialWorkspace.workspaceAria": "Espace de relecture éditoriale",
  "editorialWorkspace.returnToRevisions": "Retour à l'historique",
  "editorialWorkspace.returnToWriting": "Retour à l'écriture",
  "editorialWorkspace.closeReview": "Fermer la relecture",
  "editorialWorkspace.reviewPackagesHeading": "Paquets de relecture",
  "editorialWorkspace.editorialReviewFallback": "Relecture éditoriale",
  "editorialWorkspace.revisionStatus": "État de révision",
  "editorialWorkspace.editorMode": "Mode d'édition",
  "editorialWorkspace.modeWriting": "Écriture",
  "editorialWorkspace.modeReviewing": "Relecture",
  "editorialWorkspace.modeSuggesting": "Suggestion",
  "editorialWorkspace.reviewingFeedback": "Consultation des retours",
  "editorialWorkspace.findInManuscript": "Rechercher dans le manuscrit",
  "editorialWorkspace.manuscriptActions": "Actions sur le manuscrit",
  "editorialWorkspace.retrySaving": "Réessayer l'enregistrement",
  "editorialWorkspace.manuscriptNavAria": "Navigation dans le manuscrit",
  "editorialWorkspace.hideNav": "Masquer la navigation du manuscrit",
  "editorialWorkspace.showNav": "Afficher la navigation du manuscrit",
  "editorialWorkspace.writerBrief": "Note à l'auteur",
  "editorialWorkspace.manuscript": "Manuscrit",
  "editorialWorkspace.searchPosition": "{index} sur {count}",
  "editorialWorkspace.noMatches": "Aucun résultat",
  "editorialWorkspace.previous": "Précédent",
  "editorialWorkspace.next": "Suivant",
  "editorialWorkspace.done": "Terminé",
  "editorialWorkspace.markup": "Annotations",
  "editorialWorkspace.markupViewAria": "Vue des annotations",
  "editorialWorkspace.simpleMarkup": "Annotations simples",
  "editorialWorkspace.allMarkup": "Toutes les annotations",

  // --- EditorialWorkspace : écran des paquets de relecture ---
  "editorialWorkspace.packageTitle": "Envoyer un manuscrit en relecture",
  "editorialWorkspace.packageDescription":
    "Créez un fichier que votre éditeur peut ouvrir dans kindling. Il peut lire, suggérer des modifications et renvoyer ses retours sans compte.",
  "editorialWorkspace.reviewRound": "Tour de relecture",
  "editorialWorkspace.reviewRoundPlaceholder": "Travail de développement — septembre",
  "editorialWorkspace.reviewRoundRequired": "Saisissez un nom pour ce tour de relecture.",
  "editorialWorkspace.briefLabel": "Note pour votre éditeur",
  "editorialWorkspace.optional": "(facultatif)",
  "editorialWorkspace.briefPlaceholder": "Sur quoi souhaitez-vous que votre éditeur se concentre ?",
  "editorialWorkspace.manuscriptToInclude": "Manuscrit à inclure",
  "editorialWorkspace.entireManuscript": "Manuscrit entier",
  "editorialWorkspace.selectedChapters": "Chapitres sélectionnés",
  "editorialWorkspace.exportingPackage": "Exportation du paquet…",
  "editorialWorkspace.exportReviewPackage": "Exporter le paquet de relecture",
  "editorialWorkspace.exportShareHelp":
    "Enregistrez le fichier, puis partagez-le avec votre éditeur.",
  "editorialWorkspace.reviewRounds": "Tours de relecture",
  "editorialWorkspace.continueReview": "Reprendre une relecture",
  "editorialWorkspace.continueReviewHelp":
    "Ouvrez les retours reçus pour examiner les suggestions et renvoyer vos décisions.",
  "editorialWorkspace.noRoundsTitle": "Aucun tour de relecture pour l'instant",
  "editorialWorkspace.noRoundsBody":
    "Vos tours de relecture apparaîtront ici après l'exportation d'un paquet.",

  // --- EditorialWorkspace : aperçu des retours ---
  "editorialWorkspace.feedbackFrom": "Retours de {name}",
  "editorialWorkspace.previewDescription":
    "Intégrez les retours de votre éditeur dans le tour de relecture d'origine, puis lisez-les en regard de votre manuscrit.",
  "editorialWorkspace.feedbackInFileAria": "Retours dans ce fichier",
  "editorialWorkspace.sceneIncluded.one": "scène incluse",
  "editorialWorkspace.sceneIncluded.other": "scènes incluses",
  "editorialWorkspace.suggestion.one": "suggestion",
  "editorialWorkspace.suggestion.other": "suggestions",
  "editorialWorkspace.comment.one": "commentaire",
  "editorialWorkspace.comment.other": "commentaires",
  "editorialWorkspace.whatHappensNext": "Que se passe-t-il ensuite",
  "editorialWorkspace.nextBody":
    "Lisez les commentaires, répondez à votre éditeur, et acceptez ou refusez les suggestions. Si vous avez réécrit un passage, kindling vous aide à positionner ses retours avant d'appliquer une modification.",
  "editorialWorkspace.nextBodyPace":
    "Vous pouvez travailler les retours à votre rythme et exporter vos réponses et décisions quand vous êtes prêt.",
  "editorialWorkspace.importReviewFeedback": "Importer et consulter les retours",
  "editorialWorkspace.importHelp":
    "Votre manuscrit ne change que lorsque vous acceptez une suggestion.",
  "editorialWorkspace.reviewDetails": "Détails de la relecture",
  "editorialWorkspace.originalBrief": "Votre note initiale",

  // --- EditorialWorkspace : menu de relecture ---
  "editorialWorkspace.currentRoundAria": "Tour de relecture actuel",
  "editorialWorkspace.roundCaption": "Tour de relecture · {name}",
  "editorialWorkspace.acceptRoundAria": "Accepter les suggestions visibles dans ce tour",
  "editorialWorkspace.acceptVisible": "Accepter les suggestions visibles",
  "editorialWorkspace.rejectRoundAria": "Refuser les suggestions visibles dans ce tour",
  "editorialWorkspace.rejectVisible": "Refuser les suggestions visibles",
  "editorialWorkspace.activeSceneProse": "Texte de la scène active",
  "editorialWorkspace.acceptActiveAria":
    "Accepter les suggestions visibles sur le texte de la scène active",
  "editorialWorkspace.rejectActiveAria":
    "Refuser les suggestions visibles sur le texte de la scène active",
  "editorialWorkspace.refreshManuscript": "Actualiser le manuscrit",
  "editorialWorkspace.sendToEditor": "Envoyer à l'éditeur",
  "editorialWorkspace.exportReplies": "Exporter les réponses et décisions",

  // --- EditorialWorkspace : boîtes de dialogue de fichier et avis ---
  "editorialWorkspace.openFileTitle": "Ouvrir un fichier de relecture ou de retours kindling",
  "editorialWorkspace.openFileFilter": "Fichiers éditoriaux kindling",
  "editorialWorkspace.exportPackageTitle": "Exporter pour la relecture éditoriale",
  "editorialWorkspace.exportPackageFilter": "Relecture kindling",
  "editorialWorkspace.reviewPackageSaved":
    "Paquet de relecture enregistré dans {path}. Transmettez ce fichier à votre éditeur.",
  "editorialWorkspace.returnFeedbackTitle": "Renvoyer les retours éditoriaux",
  "editorialWorkspace.recoveryFilter": "Relecture de récupération kindling",
  "editorialWorkspace.feedbackFilter": "Retours kindling",
  "editorialWorkspace.recoverySaved":
    "Relecture de récupération enregistrée dans {path}. Ouvrez-la dans kindling pour reprendre votre travail, puis exportez les retours normalement.",
  "editorialWorkspace.feedbackSaved":
    "Retours enregistrés dans {path}. Renvoyez ce fichier à l'auteur. Vous pouvez continuer la relecture et exporter une autre réponse ultérieurement.",
  "editorialWorkspace.sendReplyTitle": "Envoyer vos réponses et décisions à votre éditeur",
  "editorialWorkspace.sendReplyFilter": "Réponse de relecture kindling",
  "editorialWorkspace.replySaved":
    "Réponse enregistrée dans {path}. Votre éditeur l'ouvre pour recevoir vos réponses et décisions tout en conservant sa relecture en cours.",
  "editorialWorkspace.feedbackImported":
    "Retours importés. Votre manuscrit n'a pas changé. Examinez les suggestions ci-dessous.",
  "editorialWorkspace.changesAccepted":
    "Modifications acceptées. Le texte précédent est conservé dans l'historique des brouillons de la scène.",
  "editorialWorkspace.decisionSaved": "Décision de relecture enregistrée.",

  // --- EditorialManuscript ---
  "editorialManuscript.currentManuscript": "Manuscrit actuel",
  "editorialManuscript.suggestedManuscript": "Manuscrit avec les modifications suggérées",
  "editorialManuscript.unlockBeforeSuggest":
    "Déverrouillez cette scène avant de suggérer des modifications à son texte.",
  "editorialManuscript.movePassage":
    "Pour déplacer ce passage, coupez-le et collez-le à la destination.",
  "editorialManuscript.marginMarker.one": "{count} commentaire ou modification dans ce paragraphe",
  "editorialManuscript.marginMarker.other":
    "{count} commentaires ou modifications dans ce paragraphe",

  "editorialManuscript.comment": "Commentaire",

  // --- ReviewSidebar : onglets, filtre et identité ---
  "reviewSidebar.feedbackAria": "Retours éditoriaux",
  "reviewSidebar.inspectorAria": "Inspecteur",
  "reviewSidebar.tabReview": "Relecture",
  "reviewSidebar.tabReferences": "Fiches",
  "reviewSidebar.showLabel": "Afficher",
  "reviewSidebar.showAria": "Afficher les retours",
  "reviewSidebar.filterOpen": "Retours en attente",
  "reviewSidebar.filterAll": "Tous les retours",
  "reviewSidebar.filterResolved": "Commentaires résolus",
  "reviewSidebar.stateResolved": "Résolu",
  "reviewSidebar.stateAccepted": "Accepté",
  "reviewSidebar.stateRejected": "Refusé",
  "reviewSidebar.stateWithdrawn": "Retiré",
  "reviewSidebar.previousFeedback": "Retour précédent",
  "reviewSidebar.nextFeedback": "Retour suivant",
  "reviewSidebar.reviewOptions": "Options de relecture",
  "reviewSidebar.yourName": "Votre nom",
  "reviewSidebar.nameShownWithFeedback": "Nom affiché avec les retours",
  "reviewSidebar.nameHelp":
    "Saisissez votre nom pour ajouter des commentaires et exporter des retours.",
  "reviewSidebar.done": "Terminé",
  "reviewSidebar.cancel": "Annuler",
  "reviewSidebar.comment": "Commentaire",
  "reviewSidebar.commentPlaceholder": "À quoi souhaitez-vous que l'auteur réfléchisse ?",
  "reviewSidebar.saveComment": "Enregistrer le commentaire",

  // --- ReviewSidebar : cartes de retours ---
  "reviewSidebar.inactiveProse": "Texte inactif",
  "reviewSidebar.suggestedEdit": "Modification suggérée",
  "reviewSidebar.authorFallback": "Relecture",
  "reviewSidebar.formattingChange": "Modification de mise en forme",
  "reviewSidebar.generalFeedback": "Retours généraux",
  "reviewSidebar.originalPassage": "Passage original",
  "reviewSidebar.currentPassage": "Passage actuel",
  "reviewSidebar.suggestedPassage": "Passage suggéré",
  "reviewSidebar.insertionPoint": "Point d'insertion",
  "reviewSidebar.emptyPassage": "Passage vide",
  "reviewSidebar.deletePassage": "Supprimer le passage",
  "reviewSidebar.conflictSuggestion":
    "Ce passage a changé. Sélectionnez l'endroit où ce retour appartient dans le manuscrit.",
  "reviewSidebar.conflictComment":
    "Ce passage a changé depuis la relecture. La discussion originale reste disponible ci-dessous.",
  "reviewSidebar.lockedHelp": "Déverrouillez cette scène pour modifier sa relecture.",
  "reviewSidebar.reply": "Répondre",
  "reviewSidebar.replyPlaceholder": "Répondre à cette conversation…",
  "reviewSidebar.reopenThread": "Rouvrir le fil",
  "reviewSidebar.resolveThread": "Résoudre le fil",
  "reviewSidebar.accept": "Accepter",
  "reviewSidebar.reject": "Refuser",
  "reviewSidebar.applyToSelection": "Appliquer au passage sélectionné",
  "reviewSidebar.reanchor": "Ré-ancrer à la sélection",
  "reviewSidebar.withdrawSuggestion": "Retirer la suggestion",
  "reviewSidebar.withdrawComment": "Retirer le commentaire",
  "reviewSidebar.emptyTitle": "Aucun retour dans cette vue",
  "reviewSidebar.emptyBody":
    "Sélectionnez un passage pour commenter, ou utilisez Suggestion pour proposer une modification.",

  // --- RevisionsPanel ---
  "revisionsPanel.title": "Historique des brouillons",
  "revisionsPanel.revisionStatus": "État de révision",
  "revisionsPanel.viewsAria": "Vues de l'historique",
  "revisionsPanel.tabHistory": "Historique des brouillons",
  "revisionsPanel.tabOverview": "Toutes les scènes",
  "revisionsPanel.loading": "Chargement de l'historique…",
  "revisionsPanel.loadFailed": "Impossible de charger l'historique.",
  "revisionsPanel.retry": "Réessayer",
  "revisionsPanel.lockedReadOnly":
    "Cette scène est verrouillée. L'historique est en lecture seule.",
  "revisionsPanel.savedDrafts": "Brouillons enregistrés",
  "revisionsPanel.draftName": "Nom du brouillon",
  "revisionsPanel.draftNamePlaceholder": "Passe post-éditeur",
  "revisionsPanel.saveNamedDraft": "Enregistrer un brouillon",
  "revisionsPanel.draftNumber": "Brouillon {number}",
  "revisionsPanel.comparisonAria": "Comparaison de brouillons",
  "revisionsPanel.keepVersionTitle": "Conserver une version de cette scène",
  "revisionsPanel.keepVersionBody":
    "Aucun brouillon enregistré pour l'instant. Enregistrez un brouillon nommé pour conserver le texte actuel de cette scène.",
  "revisionsPanel.compareWith": "Comparer avec",
  "revisionsPanel.currentProse": "Texte actuel",
  "revisionsPanel.draftOption": "Brouillon {number} · {name}",
  "revisionsPanel.restoreSelected": "Restaurer le brouillon sélectionné",
  "revisionsPanel.restoreConfirmAria": "Confirmer la restauration du brouillon",
  "revisionsPanel.restoreConfirmTitle": "Restaurer « {name} » ?",
  "revisionsPanel.restoreConfirmBody":
    "Le texte actuel sera conservé comme un autre brouillon. La structure des temps forts doit toujours correspondre.",
  "revisionsPanel.cancelRestore": "Annuler la restauration",
  "revisionsPanel.restoreAndPreserve": "Restaurer et conserver le texte actuel",
  "revisionsPanel.comparisonChanged":
    "Le texte supprimé est marqué à gauche ; le texte ajouté est marqué à droite.",
  "revisionsPanel.comparisonUnchanged": "Aucune modification de texte entre ces versions.",
  "revisionsPanel.formattingNotCompared": "La mise en forme n'est pas comparée.",
  "revisionsPanel.savedDraft": "Brouillon enregistré",
  "revisionsPanel.comparisonVersionAria": "Version de comparaison",
  "revisionsPanel.workingManuscript": "Manuscrit de travail",
  "revisionsPanel.pageProse": "Texte de page",
  "revisionsPanel.beatProse": "Texte du temps fort",
  "revisionsPanel.currentProseLabel": "le texte actuel",
  "revisionsPanel.prunedRemoved.one":
    "{names} a été supprimé car seuls les brouillons automatiques les plus récents sont conservés.",
  "revisionsPanel.prunedRemoved.other":
    "{names} ont été supprimés car seuls les brouillons automatiques les plus récents sont conservés.",
  "revisionsPanel.prunedJoin": " et ",
  "revisionsPanel.prunedComparing": "Comparaison de {before} avec {after}.",
  "revisionsPanel.beforeRestoring": "Avant la restauration de {name}",
  "revisionsPanel.chapterHeader": "Chapitre",
  "revisionsPanel.sceneHeader": "Scène",
  "revisionsPanel.revisionStatusHeader": "État de révision",
  "revisionsPanel.savingOrLoading": "Enregistrement ou chargement…",

  // --- SnapshotsPanel ---
  "snapshotsPanel.title": "Instantanés",
  "snapshotsPanel.loadFailed": "Échec du chargement des instantanés",
  "snapshotsPanel.createFailed": "Échec de la création de l'instantané",
  "snapshotsPanel.restoreFailed": "Échec de la restauration de l'instantané",
  "snapshotsPanel.deleteFailed": "Échec de la suppression de l'instantané",
  "snapshotsPanel.unsavedProse":
    "Enregistrez ou récupérez le texte non enregistré avant de restaurer un instantané.",
  "snapshotsPanel.defaultName": "Instantané {date}",
  "snapshotsPanel.projectFallback": "Projet",
  "snapshotsPanel.restoredName": "{name} (Restauré)",
  "snapshotsPanel.triggerManual": "Manuel",
  "snapshotsPanel.triggerExport": "Export",
  "snapshotsPanel.triggerAuto": "Auto",
  "snapshotsPanel.intro":
    "Un instantané est un point de restauration pour l'ensemble du projet. Créez-en un avant d'apporter des modifications importantes.",
  "snapshotsPanel.create": "Créer un instantané",
  "snapshotsPanel.newSnapshot": "Nouvel instantané",
  "snapshotsPanel.name": "Nom",
  "snapshotsPanel.namePlaceholder": "ex. Avant la réécriture du deuxième acte",
  "snapshotsPanel.description": "Description",
  "snapshotsPanel.optional": "(facultatif)",
  "snapshotsPanel.descriptionPlaceholder": "Ce qui a changé, ou pourquoi vous enregistrez ce point",
  "snapshotsPanel.cancel": "Annuler",
  "snapshotsPanel.creating": "Création de l'instantané…",
  "snapshotsPanel.created": "Instantané créé",
  "snapshotsPanel.createdBody":
    "« {name} » est maintenant le point de restauration le plus récent.",
  "snapshotsPanel.loading": "Chargement des instantanés…",
  "snapshotsPanel.loadErrorTitle": "Impossible de charger les instantanés",
  "snapshotsPanel.emptyTitle": "Aucun instantané pour l'instant",
  "snapshotsPanel.emptyBody":
    "Votre premier instantané apparaîtra ici, avec sa date et son contenu.",
  "snapshotsPanel.listAria": "Instantanés",
  "snapshotsPanel.restoreAria": "Restaurer l'instantané",
  "snapshotsPanel.restore": "Restaurer",
  "snapshotsPanel.deleteAria": "Supprimer l'instantané du {date}",
  "snapshotsPanel.restoreTitle": "Restaurer cet instantané ?",
  "snapshotsPanel.restoreMode": "Mode de restauration",
  "snapshotsPanel.replaceProject": "Remplacer le projet actuel",
  "snapshotsPanel.replaceHelp":
    "Écrase ce projet. Les scènes de l'instantané conservent leur historique ; les scènes qu'il ne contient pas sont supprimées avec le leur. Créez un instantané d'abord si vous souhaitez conserver l'état actuel.",
  "snapshotsPanel.createNewProject": "Créer un nouveau projet",
  "snapshotsPanel.createNewHelp": "Crée une copie du projet depuis cet instantané.",
  "snapshotsPanel.newProjectName": "Nom du nouveau projet",
  "snapshotsPanel.newProjectNameHelp": "Saisissez un nom pour le nouveau projet.",
  "snapshotsPanel.restoring": "Restauration…",
  "snapshotsPanel.deleteTitle": "Supprimer cet instantané ?",
  "snapshotsPanel.deleteBody":
    "Ce point de restauration sera supprimé. Votre projet n'est pas modifié.",
  "snapshotsPanel.deleting": "Suppression…",
  "snapshotsPanel.delete": "Supprimer",
  // --- Libellés d'état de révision (les valeurs stockées restent first_draft, editor_review, revised, final) ---
  "revisionStatus.first_draft": "Premier brouillon",
  "revisionStatus.editor_review": "Relecture",
  "revisionStatus.revised": "Révisée",
  "revisionStatus.final": "Final",
  "editorialWorkspace.localReviewRound": "Relecture locale",
};

export default catalog;
