import type { Catalog } from "../../types";

/** Boîte de dialogue Réglages et ses volets Apparence, Auteur, Clavier et Projet. */
const catalog: Catalog = {
  // --- Boîte de dialogue Réglages : navigation et chrome ---
  "dialog.title": "Réglages",
  "dialog.closeLabel": "Fermer les réglages",
  "dialog.areasAria": "Zones des réglages",
  "dialog.groupPreferences": "Préférences",
  "dialog.groupProjects": "Projets",
  "dialog.sectionManuscript": "Manuscrit",
  "dialog.sectionReferenceLibrary": "Bibliothèque de fiches",
  "dialog.areaAppearance": "Apparence et conseils",
  "dialog.areaAuthor": "Auteur et contact",
  "dialog.areaKeyboard": "Raccourcis clavier",
  "dialog.areaDetails": "Détails du projet",
  "dialog.areaReferences": "Types de fiches",
  "dialog.areaTags": "Étiquettes",
  "dialog.areaFields": "Champs personnalisés",
  "dialog.discardSettings": "Annuler les modifications de réglages non enregistrées ?",
  "dialog.discardProjectSwitch":
    "Annuler les modifications non enregistrées pour ce projet avant de changer ?",
  "dialog.keepEditing": "Continuer à modifier",
  "dialog.discard": "Annuler les modifications",
  "dialog.loadingProjects": "Chargement des projets…",
  "dialog.retryLoadProjects": "Réessayer le chargement des projets",
  "dialog.noProjects": "Aucun projet disponible.",
  "dialog.projectLabel": "Projet",
  "dialog.projectHelp": "Réglages pour ce projet. Votre manuscrit ouvert reste en place.",
  "dialog.appliesToAll": "S'applique à tous les projets.",
  "dialog.noProjectsHelp":
    "Aucun projet pour l'instant. Créez ou importez un projet pour configurer ses réglages.",
  "dialog.loadProjectsFailed": "Impossible de charger les projets : {error}",

  // --- Apparence ---
  "appearance.lede":
    "Choisissez l'apparence de kindling et la quantité de conseils qu'il affiche. Les modifications s'appliquent immédiatement à tous les projets.",
  "appearance.title": "Apparence",
  "appearance.themeAria": "Thème",
  "appearance.themeDark": "Sombre",
  "appearance.themeLight": "Clair",
  "appearance.themeSystem": "Système",
  "appearance.systemHelp":
    "« Système » suit le réglage d'apparence de votre système d'exploitation.",
  "appearance.language": "Langue",
  "appearance.languageSystem": "Langue du système",
  "appearance.languageHelp":
    "« Langue du système » suit la langue de votre système d'exploitation. Les menus changent aussi.",
  "appearance.guidance": "Conseils",
  "appearance.showTips": "Afficher les conseils de guidage",
  "appearance.tipsHelp":
    "Un conseil apparaît la première fois que vous visitez la barre latérale, le volet de scène et les fiches. Chaque conseil s'affiche une seule fois.",
  "appearance.showTipsAgain": "Afficher les conseils à nouveau",
  "appearance.takeTour": "Faire la visite",
  "appearance.turnOnTips": "Activez les conseils de guidage pour les afficher à nouveau.",
  "appearance.tipsReset":
    "Les conseils apparaîtront à nouveau au fur et à mesure que vous visitez chaque zone.",

  // --- Auteur et contact ---
  "author.lede":
    "Vos coordonnées d'auteur et de contact s'appliquent aux pages de titre des manuscrits de tous les projets.",
  "author.loading": "Chargement des coordonnées de l'auteur…",
  "author.infoTitle": "Informations sur l'auteur",
  "author.nameLabel": "Nom de l'auteur",
  "author.namePlaceholder": "Votre nom légal",
  "author.nameHelp":
    "Utilisé dans les coordonnées sur les pages de titre. Les projets peuvent remplacer cela par un pseudonyme.",
  "author.contactTitle": "Coordonnées",
  "author.contactHelp":
    "Coordonnées facultatives pour les pages de titre des manuscrits. Utilisez le format qui convient à votre pays.",
  "author.address1": "Ligne d'adresse 1",
  "author.address1Placeholder": "Adresse postale ou boîte postale",
  "author.address2": "Ligne d'adresse 2",
  "author.address2Placeholder": "Ville, département, code postal, pays",
  "author.phone": "Téléphone",
  "author.phonePlaceholder": "+33 6 12 34 56 78",
  "author.email": "E-mail",
  "author.emailPlaceholder": "auteur@email.com",
  "author.saved": "Coordonnées de l'auteur enregistrées.",
  "author.save": "Enregistrer les coordonnées de l'auteur",
  "author.loadFailedTitle": "Impossible de charger les coordonnées de l'auteur",
  "author.loadFailedHelp":
    "Si le fichier de réglages est endommagé, réinitialisez les coordonnées de l'auteur pour restaurer l'enregistrement et les exports. Une copie de récupération du fichier original sera conservée.",
  "author.retry": "Réessayer le chargement des coordonnées de l'auteur",
  "author.reset": "Réinitialiser les coordonnées de l'auteur",

  // --- Raccourcis clavier ---
  "keyboard.lede":
    "Choisissez un raccourci, puis appuyez sur une nouvelle combinaison de touches. Les modifications s'appliquent immédiatement à tous les projets. Échap annule l'enregistrement ; Tab passe au contrôle suivant.",
  "keyboard.systemHelp":
    "L'édition de texte standard et les contrôles système (comme copier, coller, annuler, Tab et Échap) conservent leurs touches habituelles.",
  "keyboard.retryLoad": "Réessayer le chargement des raccourcis",
  "keyboard.filterLabel": "Filtrer les commandes",
  "keyboard.resetDefaults": "Rétablir tous les défauts",
  "keyboard.loading": "Chargement des raccourcis…",
  "keyboard.shortcutFor": "Raccourci pour {label}",
  "keyboard.pressKeys": "Appuyez sur les touches…",
  "keyboard.unassigned": "Non assigné",
  "keyboard.clearFor": "Effacer le raccourci pour {label}",
  "keyboard.clear": "Effacer",
  "keyboard.noMatches": "Aucune commande correspondante.",
  "keyboard.saved": "Raccourcis clavier enregistrés.",
  "keyboard.saveFailed": "Impossible d'enregistrer les raccourcis clavier : {error}",
  "keyboard.invalidKey":
    "Utilisez Commande sur macOS ou Ctrl sur Windows/Linux, avec une lettre, un chiffre, une touche de ponctuation ou F1–F24.",

  // --- Détails du projet ---
  "project.ledePrefix": "Les modifications ici s'appliquent uniquement à ",
  "project.ledeSuffix": ".",
  "project.tagChanges": "Les modifications d'étiquettes sont enregistrées au fur et à mesure.",
  "project.fieldChanges":
    "Les modifications de champs personnalisés sont enregistrées au fur et à mesure. Les champs sont listés pour les types de fiches enregistrés et activés.",
  "project.optional": "(facultatif)",
  "project.penName": "Pseudonyme",
  "project.penNamePlaceholder": "Laissez vide pour utiliser votre nom d'auteur",
  "project.penNameHelp":
    "Si fourni, ce nom sera utilisé comme signature sur les pages de titre à la place de votre nom d'auteur.",
  "project.genre": "Genre",
  "project.genrePlaceholder": "ex. Fiction littéraire, Science-fiction, Policier",
  "project.genreHelp": "Le genre sera affiché sur les pages de titre des manuscrits.",
  "project.description": "Description du projet",
  "project.descriptionPlaceholder": "Court résumé ou notes sur ce projet",
  "project.dailyGoal": "Objectif d'écriture quotidien",
  "project.loadDailyGoalFailed": "Impossible de charger l'objectif quotidien : {error}",
  "project.dailyGoalHelp":
    "Mots nets ajoutés par jour dans ce projet. Définissez 0 pour désactiver l'objectif. Les modifications s'appliquent aujourd'hui ; les jours de série précédents conservent leurs objectifs d'origine.",
  "project.wordTarget": "Objectif de mots",
  "project.wordTargetPlaceholder": "ex. 80000",
  "project.referencesHelp":
    "Choisissez quels types de fiches apparaissent dans le volet des fiches de ce projet. La désactivation d'un type conserve ses entrées existantes.",
  "project.enableRefTypes": "Activez des types de fiches pour configurer des champs personnalisés.",
  "project.saved": "Modifications du projet enregistrées.",
  "project.save": "Enregistrer les modifications du projet",
  "project.wordTargetInvalid": "L'objectif de mots doit être un nombre entier non négatif",
  "project.dailyGoalInvalid":
    "L'objectif quotidien doit être un nombre entier compris entre 0 et 1 000 000",
  "project.disabledDraft":
    "Enregistrez ou annulez le brouillon du champ personnalisé {label} avant de désactiver ce type de fiche.",
};

export default catalog;
