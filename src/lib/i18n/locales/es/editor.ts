import type { Catalog } from "../../types";

/** Scene panel, beat list, prose editors and writing statistics. */
const catalog: Catalog = {
  // ScenePanel
  "scenePanel.revisions": "Revisiones",
  "scenePanel.openingRevisions": "Abriendo revisiones…",
  "scenePanel.revisionsUnsavedError":
    "Guarda o recupera la prosa sin guardar antes de abrir Revisiones.",
  "scenePanel.viewGroupLabel": "Vista",
  "scenePanel.beatsTab": "Tiempos",
  "scenePanel.beatsTabTooltip": "Tarjetas de tiempos",
  "scenePanel.pageTab": "Página",
  "scenePanel.pageTabTooltip": "Prosa a página completa",
  "scenePanel.sequence": "Secuencia",
  "scenePanel.chapter": "Capítulo",
  "scenePanel.breadcrumb": "{kind} · {name}",
  "scenePanel.pageEstimate": "~{pages} páginas",
  "scenePanel.locked": "Bloqueado",
  "scenePanel.sceneTypeLabel": "Tipo de escena",
  "scenePanel.sceneTypeNormal": "Normal",
  "scenePanel.sceneTypeNotes": "Notas",
  "scenePanel.sceneTypeTodo": "Por hacer",
  "scenePanel.sceneTypeUnused": "Sin usar",
  "scenePanel.statusLabel": "Estado",
  "scenePanel.sceneStatusDraft": "Borrador",
  "scenePanel.sceneStatusRevised": "Revisado",
  "scenePanel.sceneStatusFinal": "Final",
  "scenePanel.planningLabel": "Planificación",
  "scenePanel.planningFixed": "Fija",
  "scenePanel.planningFlexible": "Flexible",
  "scenePanel.planningUndefined": "Indefinida",
  "scenePanel.planningTooltip":
    "Controla cuánta estructura tiene esta escena: Indefinida → Flexible → Fija",
  "scenePanel.aboutPlanning": "Acerca de la planificación",
  "scenePanel.planningHelp":
    "Controla cuánta estructura tiene esta escena: Indefinida, Flexible o Fija.",
  "scenePanel.tagsLabel": "Etiquetas",
  "scenePanel.rollingOutline": "Esquema continuo",
  "scenePanel.dismiss": "Descartar",
  "scenePanel.guideLedeBefore": "El menú ",
  "scenePanel.guidePlanning": "Planificación",
  "scenePanel.guideLedeAfter":
    " superior controla cuánta estructura tiene esta escena. Úsalo para desarrollar tu historia gradualmente:",
  "scenePanel.guideUndefinedDesc":
    "Un marcador de posición — sabes que existe pero no la has planificado.",
  "scenePanel.guideFlexibleDesc": "Tienes la idea general — una sinopsis y dirección aproximada.",
  "scenePanel.guideFixedDesc": "Estructura completa con tiempos, referencias y notas.",
  "scenePanel.undefinedSceneTitle": "Escena indefinida",
  "scenePanel.undefinedSceneBody":
    "Esto es un marcador de posición — sabes que existe pero aún no la has planificado.",
  "scenePanel.undefinedSceneHelp":
    "Añade una sinopsis a continuación para capturar la idea, y luego promuévela cuando estés listo para desarrollarla.",
  "scenePanel.switchToFlexible": "Cambiar a Flexible",
  "scenePanel.goStraightToFixed": "Ir directo a Fija",
  "scenePanel.flexibleSceneTitle": "Escena flexible",
  "scenePanel.flexibleSceneBody":
    "Tienes una idea para esta escena pero no has definido la estructura.",
  "scenePanel.flexibleSceneHelp":
    "Usa la sinopsis para plasmar tu intención. Cuando estés listo para dividirla en tiempos, cambia a Fija.",
  "scenePanel.defineBeats": "Definir tiempos",
  "scenePanel.sceneLockedTitle": "Esta escena está bloqueada",
  "scenePanel.chapterLockedHelp":
    "El capítulo superior está bloqueado. Desbloquea el capítulo para editar esta escena.",
  "scenePanel.sceneLockedHelp": "Desbloquea esta escena desde el panel lateral para hacer cambios.",
  "scenePanel.synopsisTitle": "Sinopsis",
  "scenePanel.editSynopsis": "Editar sinopsis",
  "scenePanel.synopsisPlaceholder": "Escribe una breve sinopsis para esta escena…",
  "scenePanel.saving": "Guardando…",
  "scenePanel.synopsisEditingHelp":
    "Presiona Escape para cerrar. Los cambios se guardan automáticamente.",
  "scenePanel.addSynopsis": "Añadir sinopsis",
  "scenePanel.sceneLockedShort": "La escena está bloqueada",
  "scenePanel.synopsisSaveFailed":
    "Sinopsis no guardada: {error}. Tu borrador se mantiene para reintentar.",
  "scenePanel.retrySynopsisSave": "Reintentar guardar sinopsis",
  "scenePanel.retrySaving": "Reintentar guardar",
  "scenePanel.unsavedChanges": "Cambios sin guardar",
  "scenePanel.referencesTitle": "Referencias",
  "scenePanel.loading": "Cargando…",
  "scenePanel.noReferences":
    "Aún no hay referencias enlazadas. Enlázalas desde el panel de referencias.",
  "scenePanel.discoveryNotesTitle": "Notas de descubrimiento",
  "scenePanel.hide": "Ocultar",
  "scenePanel.show": "Mostrar",
  "scenePanel.newNoteLabel": "Nueva nota de descubrimiento",
  "scenePanel.newNotePlaceholder": "¿Qué descubriste?",
  "scenePanel.addingNote": "Añadiendo…",
  "scenePanel.addNote": "Añadir nota",
  "scenePanel.cancel": "Cancelar",
  "scenePanel.editNoteLabel": "Editar nota de descubrimiento",
  "scenePanel.save": "Guardar",
  "scenePanel.edit": "Editar",
  "scenePanel.promoting": "Promoviendo…",
  "scenePanel.promoteToBeat": "Promover a tiempo",
  "scenePanel.delete": "Eliminar",
  "scenePanel.noNotes":
    "No hay notas de descubrimiento aún. Captura lo que encuentres mientras escribes — las notas no forman parte del manuscrito.",
  "scenePanel.contentTitle": "Contenido",
  "scenePanel.referenceLoadError": "Error al cargar elementos de referencia de la escena",
  "scenePanel.metadataUpdateError": "Error al actualizar los metadatos de la escena",
  "scenePanel.planningUpdateError": "Error al actualizar el estado de planificación",
  "scenePanel.renameSceneError": "Error al renombrar la escena: {error}",
  "scenePanel.addNoteError": "Error al añadir nota de descubrimiento: {error}",
  "scenePanel.saveNoteError": "Error al guardar nota de descubrimiento: {error}",
  "scenePanel.deleteNoteError": "Error al eliminar nota de descubrimiento: {error}",
  "scenePanel.promoteNoteError": "Error al convertir nota en tiempo: {error}",
  "scenePanel.modeSwitchUnsavedError":
    "Guarda, recupera o descarta la prosa sin guardar en Buscar y Reemplazar antes de cambiar de modo de editor.",
  "scenePanel.modeSwitchError": "Error al cambiar el modo de editor: {error}",
  "scenePanel.sceneChangedError":
    "El proyecto o escena seleccionada cambió. Vuelve a abrir Buscar y Reemplazar.",
  "scenePanel.emptyTitle": "Selecciona una escena para empezar a escribir",
  "scenePanel.emptyBody": "Elige una escena en el panel lateral para ver su contenido.",
  "scenePanel.switchToBeatTitle": "Cambiar a Vista de Tiempos",
  "scenePanel.switchToBeatMessage":
    "Cualquier cambio realizado en la Vista de Página se sincronizará con los tiempos correspondientes. ¿Continuar?",
  "scenePanel.switchConfirm": "Cambiar",

  // BeatView
  "beatView.beatsTitle": "Tiempos",
  "beatView.addBeat": "Añadir tiempo",
  "beatView.dragToReorder": "Arrastrar para reordenar",
  "beatView.beatTitleLabel": "Título del tiempo",
  "beatView.beatWords": "{count} palabras",
  "beatView.beatMenu": "Menú del tiempo",
  "beatView.sceneLocked": "La escena está bloqueada",
  "beatView.prosePlaceholder": "Escribe la prosa para este tiempo…",
  "beatView.openBeatProse": "Abrir prosa del tiempo {number}",
  "beatView.emptyTitle": "Aún no hay tiempos",
  "beatView.emptyBody":
    "Los tiempos provienen de un esquema importado, o puedes añadirlos aquí momento a momento.",
  "beatView.addFirstBeat": "Añade tu primer tiempo",
  "beatView.newBeatLabel": "Nuevo tiempo",
  "beatView.newBeatPlaceholder": "Describe lo que ocurre en este tiempo…",
  "beatView.newBeatHelp": "Presiona Enter para crear, Escape para cancelar.",
  "beatView.cancel": "Cancelar",
  "beatView.creating": "Creando…",
  "beatView.createBeat": "Crear tiempo",
  "beatView.deleteTitle": "Eliminar tiempo",
  "beatView.deleteFirstMessage":
    "¿Seguro que quieres eliminar este tiempo? Este es el primer tiempo, así que su prosa se eliminará permanentemente.",
  "beatView.deleteMergeMessage":
    "¿Seguro que quieres eliminar este tiempo? Cualquier prosa se fusionará con el tiempo anterior.",
  "beatView.menuRename": "Renombrar",
  "beatView.menuSplit": "Dividir en el cursor",
  "beatView.menuMerge": "Fusionar con el siguiente",
  "beatView.menuMoveUp": "Mover arriba",
  "beatView.menuMoveDown": "Mover abajo",
  "beatView.menuDelete": "Eliminar",
  "beatView.createError": "Error al crear el tiempo: {error}",
  "beatView.renameError": "Error al renombrar el tiempo: {error}",
  "beatView.deleteError": "Error al eliminar el tiempo: {error}",
  "beatView.splitError": "Error al dividir el tiempo: {error}",
  "beatView.mergeError": "Error al fusionar tiempos: {error}",
  "beatView.transformUnsavedError":
    "Primero guarda los tiempos afectados o recupera sus borradores sin guardar en Buscar y Reemplazar.",
  "beatView.reorderError": "Error al reordenar los tiempos: {error}",
  "beatView.moveUpAnnouncement":
    "Tiempo “{title}” movido arriba, a la posición {position} de {total}.",
  "beatView.moveDownAnnouncement":
    "Tiempo “{title}” movido abajo, a la posición {position} de {total}.",

  // ProseToolbar
  "proseToolbar.label": "Formato de texto",
  "proseToolbar.bold": "Negrita",
  "proseToolbar.italic": "Cursiva",
  "proseToolbar.underline": "Subrayado",
  "proseToolbar.monospace": "Monoespaciado",
  "proseToolbar.alignLeft": "Alinear a la izquierda",
  "proseToolbar.alignCenter": "Centrar",
  "proseToolbar.alignRight": "Alinear a la derecha",
  "proseToolbar.alignJustify": "Justificar",
  "proseToolbar.blockquote": "Cita en bloque",
  "proseToolbar.indent": "Sangría",
  "proseToolbar.undo": "Deshacer",
  "proseToolbar.redo": "Rehacer",

  // SluglineInput
  "sluglineInput.sceneHeading": "Encabezado de escena",
  "sluglineInput.interiorOrExterior": "Interior o exterior",
  "sluglineInput.location": "Ubicación",
  "sluglineInput.locationPlaceholder": "Ubicación",
  "sluglineInput.timeOfDay": "Momento del día",

  // WritingStatusBar
  "writingStatusBar.statisticsTitle": "Estadísticas de escritura",
  "writingStatusBar.hideStatistics": "Ocultar estadísticas",
  "writingStatusBar.totalWords": "Total de palabras",
  "writingStatusBar.scenesWithProse": "Escenas con prosa",
  "writingStatusBar.emptyScenes": "Escenas vacías",
  "writingStatusBar.averageWordsPerScene": "Promedio de palabras por escena",
  "writingStatusBar.savedProseNote":
    "Solo prosa guardada. El contenido archivado está excluido; el promedio incluye escenas vacías.",
  "writingStatusBar.wordsPerChapter": "Palabras por capítulo",
  "writingStatusBar.chapterColumn": "Capítulo",
  "writingStatusBar.wordsColumn": "Palabras",
  "writingStatusBar.noChapters": "Aún no hay capítulos.",
  "writingStatusBar.sceneWords": "Escena: {count} palabras",
  "writingStatusBar.chapterWords": "Capítulo: {count} palabras",
  "writingStatusBar.projectWords": "Proyecto: {count} palabras",
  "writingStatusBar.sessionWords": "Sesión: {count} palabras",
  "writingStatusBar.sessionTooltip":
    "Palabras netas guardadas desde que se abrió la aplicación o se restableció la sesión.",

  // WritingProgress
  "writingProgress.projectWords": "{count} palabras en el proyecto",
  "writingProgress.todayTooltip":
    "Palabras netas añadidas a través de ediciones guardadas hoy. Las eliminaciones reducen este total.",
  "writingProgress.todayWithGoal": "Hoy: {today} / {goal} palabras",
  "writingProgress.todayWithoutGoal": "Hoy: {today} palabras · meta desactivada",
  "writingProgress.dailyGoal": "Meta diaria de escritura",
  "writingProgress.sessionWords": "Sesión: {count} palabras",
  "writingProgress.sessionTooltip":
    "Palabras netas guardadas en este proyecto desde que se abrió la aplicación o se restableció.",
  "writingProgress.resetSession": "Restablecer sesión de escritura",
  "writingProgress.reset": "Restablecer",
  "writingProgress.streak.one": "Racha de escritura de {count} día",
  "writingProgress.streak.many": "Racha de escritura de {count} días",
  "writingProgress.resetError": "Guarda tu prosa pendiente antes de restablecer: {error}",

  // NovelEditor
  "novelEditor.placeholder": "Escribe tu prosa…",
  "novelEditor.sceneProse": "Prosa de la escena",
  "novelEditor.beatProse": "Prosa del tiempo",
  "novelEditor.saving": "Guardando…",
  "novelEditor.saveError": "No se pudo guardar — tu texto sigue aquí",

  // PageView
  "pageView.title": "Prosa de la escena",
  "pageView.words": "{count} palabras",
  "pageView.lockedPlaceholder": "La escena está bloqueada",
  "pageView.placeholder": "Escribe aquí la prosa de tu escena…",

  // Previously
  "previously.toggle": "Anteriormente",
  "previously.toolsLabel": "Herramientas de escena",
  "previously.heading": "Anteriormente",
  "previously.loadError": "No se pudo cargar el contexto de la escena anterior.",
  "previously.retry": "Reintentar",
};

export default catalog;
