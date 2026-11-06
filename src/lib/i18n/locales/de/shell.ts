import type { Catalog } from "../../types";

/** App shell: quit flow, save failures, toasts and dialog chrome. */
const catalog: Catalog = {
  quitReviewTitle: "Ohne Speichern der Lektoratsänderungen beenden?",
  quitWritingTitle: "Ohne Speichern der Schreibänderungen beenden?",
  quitSynopsisTitle: "Ohne Speichern der Zusammenfassungsänderungen beenden?",
  quitReviewMessage:
    "Ihr Lektorat konnte nicht gespeichert werden. Bleiben Sie zum Erneuten Versuchen oder exportieren Sie eine Wiederherstellungskopie. Beim Beenden und Verwerfen werden alle nicht gespeicherten Lektorats-, Text- und Zusammenfassungsänderungen entfernt.",
  quitWritingMessage:
    "Einige Text- oder Zusammenfassungsänderungen konnten nicht gespeichert werden. Beenden und nicht gespeicherte Änderungen verwerfen oder weiter bearbeiten und erneut versuchen.",
  quitSynopsisMessage:
    "Einige Zusammenfassungsänderungen konnten nicht gespeichert werden. Beenden und nicht gespeicherte Änderungen verwerfen oder weiter bearbeiten und erneut versuchen.",
  quitAndDiscard: "Beenden und verwerfen",
  keepEditing: "Weiter bearbeiten",
  couldNotQuit: "Beenden nicht möglich: {error}",
  sceneUnavailable: "Diese Szene ist nicht mehr verfügbar.",
  proseUnsaved:
    "Textänderungen konnten nicht gespeichert werden. Nicht gespeicherte Entwürfe in Suchen und Ersetzen prüfen.",
  synopsisUnsaved: "Ihre Zusammenfassungsänderungen wurden nicht gespeichert.",
  retryAllSynopsisSaves: "Alle Zusammenfassungsspeicherungen erneut versuchen",
  retrySaving: "Erneut versuchen",
  savingShort: "Wird gespeichert…",
  dismissError: "Fehler schließen",
  dismiss: "Schließen",
  closeConfirmation: "Bestätigung schließen",
};

export default catalog;
