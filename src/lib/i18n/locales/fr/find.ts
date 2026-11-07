import type { Catalog } from "../../types";

/** Boîte de dialogue Rechercher et remplacer. */
const catalog: Catalog = {
  "findReplaceDialog.title": "Rechercher et remplacer",
  "findReplaceDialog.find": "Rechercher",
  "findReplaceDialog.previousMatch": "Résultat précédent (Maj+Entrée)",
  "findReplaceDialog.previous": "Précédent",
  "findReplaceDialog.nextMatch": "Résultat suivant (Entrée)",
  "findReplaceDialog.next": "Suivant",
  "findReplaceDialog.loadingProse": "Chargement du texte…",
  "findReplaceDialog.enterQuery": "Saisissez le texte à rechercher.",
  "findReplaceDialog.searchIn": "Rechercher dans",
  "findReplaceDialog.currentScene": "La scène actuelle",
  "findReplaceDialog.entireProject": "Le projet entier",
  "findReplaceDialog.matchCase": "Respecter la casse",
  "findReplaceDialog.wholeWords": "Mots entiers",
  "findReplaceDialog.replace": "Remplacer",
  "findReplaceDialog.replaceWith": "Remplacer par",
  "findReplaceDialog.scopeHelp":
    "Recherche dans le texte visible des scènes fixées. Les scènes flexibles, non définies et archivées sont exclues ; les scènes verrouillées sont consultables mais non remplaçables.",
  "findReplaceDialog.position": "{current} sur {total}",
  "findReplaceDialog.locked": " · Verrouillée",
  "findReplaceDialog.unsavedDraft": " · Brouillon non enregistré",
  "findReplaceDialog.openScene": "Ouvrir la scène",
  "findReplaceDialog.noMoreMatches":
    "Plus aucun résultat. Utilisez Suivant ou Précédent pour continuer.",
  "findReplaceDialog.noMatches": "Aucun résultat.",
  "findReplaceDialog.confirmReplaceAll": "Confirmer le remplacement global",
  "findReplaceDialog.retryLoading": "Réessayer le chargement",
  "findReplaceDialog.retrySavingDrafts": "Réessayer l'enregistrement des brouillons",
  "findReplaceDialog.draftsHelp":
    "Les brouillons non enregistrés sont conservés pour cette session, même après la fermeture du projet. Les documents verrouillés ou manquants et les erreurs d'enregistrement non reconnues ne sont pas relancés automatiquement et ne bloquent pas les autres scènes. Vous pouvez réessayer après avoir résolu l'erreur, ou copier ces brouillons avant de les supprimer. Les occurrences dans les documents avec des brouillons non enregistrés ne peuvent pas être remplacées.",
  "findReplaceDialog.unsavedScene": "Scène non enregistrée {index}",
  "findReplaceDialog.unsavedBeat": "Temps fort non enregistré {index}",
  "findReplaceDialog.draftText": "Texte du brouillon (sélectionnez pour copier)",
  "findReplaceDialog.discardQuestion":
    "Supprimer ces brouillons non enregistrés ? Leurs modifications seront perdues. Copiez d'abord tout texte que vous souhaitez conserver.",
  "findReplaceDialog.confirmDiscardDrafts": "Confirmer la suppression des brouillons",
  "findReplaceDialog.keepDrafts": "Conserver les brouillons",
  "findReplaceDialog.discardDrafts": "Supprimer les brouillons non enregistrés…",
  "findReplaceDialog.undoReplacement": "Annuler le remplacement",
  "findReplaceDialog.replaceAll": "Tout remplacer",
  "findReplaceDialog.replaceMatch": "Remplacer l'occurrence",
  "findReplaceDialog.replacementUndone": "Remplacement annulé.",
  "findReplaceDialog.replacementSaved": "Remplacement enregistré.",
  "findReplaceDialog.noChangesNeeded": "Aucune modification nécessaire.",
  "findReplaceDialog.confirmReplaceProject.one":
    "Remplacer {count} occurrence dans le projet entier ?",
  "findReplaceDialog.confirmReplaceProject.other":
    "Remplacer {count} occurrences dans le projet entier ?",
  "findReplaceDialog.confirmReplaceScene.one":
    "Remplacer {count} occurrence dans la scène actuelle ?",
  "findReplaceDialog.confirmReplaceScene.other":
    "Remplacer {count} occurrences dans la scène actuelle ?",
  "findReplaceDialog.confirmSkipped.one":
    "{count} occurrence verrouillée ou non enregistrée sera ignorée.",
  "findReplaceDialog.confirmSkipped.other":
    "{count} occurrences verrouillées ou non enregistrées seront ignorées.",
};

export default catalog;
