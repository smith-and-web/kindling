import type { Catalog } from "../../types";

/** App shell : flux de fermeture, échecs d'enregistrement, notifications et chrome des boîtes de dialogue. */
const catalog: Catalog = {
  quitReviewTitle: "Quitter sans enregistrer les modifications de relecture ?",
  quitWritingTitle: "Quitter sans enregistrer les modifications d'écriture ?",
  quitSynopsisTitle: "Quitter sans enregistrer les modifications du synopsis ?",
  quitReviewMessage:
    "Votre relecture n'a pas pu être enregistrée. Continuez à modifier pour réessayer ou exportez une copie de récupération. En quittant et annulant, toutes les modifications non enregistrées de relecture, de texte et de synopsis seront perdues.",
  quitWritingMessage:
    "Certaines modifications de texte ou de synopsis n'ont pas pu être enregistrées. Quittez et annulez ces modifications d'écriture non enregistrées, ou continuez à modifier pour réessayer l'enregistrement.",
  quitSynopsisMessage:
    "Certaines modifications de synopsis n'ont pas pu être enregistrées. Quittez et annulez ces modifications de synopsis non enregistrées, ou continuez à modifier pour réessayer l'enregistrement.",
  quitAndDiscard: "Quitter et annuler",
  keepEditing: "Continuer à modifier",
  couldNotQuit: "Impossible de quitter : {error}",
  sceneUnavailable: "Cette scène n'est plus disponible.",
  proseUnsaved:
    "Les modifications du texte n'ont pas pu être enregistrées. Consultez les brouillons non enregistrés dans Rechercher et remplacer.",
  synopsisUnsaved: "Vos modifications du synopsis n'ont pas été enregistrées.",
  retryAllSynopsisSaves: "Réessayer tous les enregistrements de synopsis",
  retrySaving: "Réessayer l'enregistrement",
  savingShort: "Enregistrement…",
  dismissError: "Fermer l'erreur",
  dismiss: "Fermer",
  closeConfirmation: "Fermer la confirmation",
};

export default catalog;
