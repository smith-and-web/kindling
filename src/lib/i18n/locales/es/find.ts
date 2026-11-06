import type { Catalog } from "../../types";

/** Find and Replace dialog. */
const catalog: Catalog = {
  "findReplaceDialog.title": "Buscar y Reemplazar",
  "findReplaceDialog.find": "Buscar",
  "findReplaceDialog.previousMatch": "Coincidencia anterior (Mayús+Enter)",
  "findReplaceDialog.previous": "Anterior",
  "findReplaceDialog.nextMatch": "Siguiente coincidencia (Enter)",
  "findReplaceDialog.next": "Siguiente",
  "findReplaceDialog.loadingProse": "Cargando prosa…",
  "findReplaceDialog.enterQuery": "Introduce el texto a buscar.",
  "findReplaceDialog.searchIn": "Buscar en",
  "findReplaceDialog.currentScene": "Escena actual",
  "findReplaceDialog.entireProject": "Todo el proyecto",
  "findReplaceDialog.matchCase": "Coincidir mayúsculas y minúsculas",
  "findReplaceDialog.wholeWords": "Palabras completas",
  "findReplaceDialog.replace": "Reemplazar",
  "findReplaceDialog.replaceWith": "Reemplazar con",
  "findReplaceDialog.scopeHelp":
    "Busca prosa visible en escenas Fijas. Las escenas Flexibles, Indefinidas y archivadas están excluidas; las escenas bloqueadas se pueden buscar pero no reemplazar.",
  "findReplaceDialog.position": "{current} de {total}",
  "findReplaceDialog.locked": " · Bloqueado",
  "findReplaceDialog.unsavedDraft": " · Borrador sin guardar",
  "findReplaceDialog.openScene": "Abrir escena",
  "findReplaceDialog.noMoreMatches":
    "No hay más coincidencias en adelante. Usa Siguiente o Anterior para continuar.",
  "findReplaceDialog.noMatches": "No se encontraron coincidencias.",
  "findReplaceDialog.confirmReplace":
    "¿Reemplazar {count} coincidencias en {scope}? Se omitirán {skipped} coincidencias bloqueadas o sin guardar.",
  "findReplaceDialog.scopeEntireProject": "todo el proyecto",
  "findReplaceDialog.scopeCurrentScene": "la escena actual",
  "findReplaceDialog.confirmReplaceAll": "Confirmar reemplazar todo",
  "findReplaceDialog.retryLoading": "Reintentar cargar",
  "findReplaceDialog.retrySavingDrafts": "Reintentar guardar borradores",
  "findReplaceDialog.draftsHelp":
    "Los borradores sin guardar se conservan para esta sesión, incluso después de cerrar el proyecto. Los documentos bloqueados o faltantes y los errores de guardado no reconocidos no se reintentan automáticamente ni bloquean otras escenas. Puedes reintentarlo tras resolver el error o copiar estos borradores antes de descartarlos. No se pueden reemplazar las coincidencias en documentos con borradores sin guardar.",
  "findReplaceDialog.unsavedScene": "Escena sin guardar {index}",
  "findReplaceDialog.unsavedBeat": "Tiempo sin guardar {index}",
  "findReplaceDialog.draftText": "Texto del borrador (seleccionar para copiar)",
  "findReplaceDialog.discardQuestion":
    "¿Descartar estos borradores sin guardar? Se perderán los cambios. Copia primero cualquier texto que quieras conservar.",
  "findReplaceDialog.confirmDiscardDrafts": "Confirmar descartar borradores",
  "findReplaceDialog.keepDrafts": "Conservar borradores",
  "findReplaceDialog.discardDrafts": "Descartar borradores sin guardar…",
  "findReplaceDialog.undoReplacement": "Deshacer reemplazo",
  "findReplaceDialog.replaceAll": "Reemplazar todo",
  "findReplaceDialog.replaceMatch": "Reemplazar coincidencia",
  "findReplaceDialog.replacementUndone": "Reemplazo deshecho.",
  "findReplaceDialog.replacementSaved": "Reemplazo guardado.",
  "findReplaceDialog.noChangesNeeded": "No se necesitan cambios.",
};

export default catalog;
