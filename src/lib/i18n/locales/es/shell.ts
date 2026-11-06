import type { Catalog } from "../../types";

/** App shell: quit flow, save failures, toasts and dialog chrome. */
const catalog: Catalog = {
  quitReviewTitle: "¿Salir sin guardar los cambios de revisión?",
  quitWritingTitle: "¿Salir sin guardar los cambios de escritura?",
  quitSynopsisTitle: "¿Salir sin guardar los cambios de sinopsis?",
  quitReviewMessage:
    "Tu revisión no se pudo guardar. Sigue editando para reintentar o exporta una copia de recuperación. Al salir y descartar se eliminarán todos los cambios no guardados de revisión, prosa y sinopsis.",
  quitWritingMessage:
    "Algunos cambios de prosa o sinopsis no se pudieron guardar. Sal y descarta estos cambios de escritura no guardados, o sigue editando para reintentar guardarlos.",
  quitSynopsisMessage:
    "Algunos cambios de sinopsis no se pudieron guardar. Sal y descarta estos cambios de sinopsis no guardados, o sigue editando para reintentar guardarlos.",
  quitAndDiscard: "Salir y descartar",
  keepEditing: "Seguir editando",
  couldNotQuit: "No se pudo salir: {error}",
  sceneUnavailable: "Esta escena ya no está disponible.",
  proseUnsaved:
    "Los cambios en la prosa no se pudieron guardar. Revisa los borradores sin guardar en Buscar y Reemplazar.",
  synopsisUnsaved: "Tus cambios en la sinopsis no se han guardado.",
  retryAllSynopsisSaves: "Reintentar guardar todas las sinopsis",
  retrySaving: "Reintentar guardar",
  savingShort: "Guardando…",
  dismissError: "Descartar error",
  dismiss: "Descartar",
  closeConfirmation: "Confirmación de cierre",
};

export default catalog;
