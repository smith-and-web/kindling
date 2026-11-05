import type { Catalog } from "../../types";

/** Copy produced outside components: stores, utils, import formats and command labels. */
const catalog: Catalog = {
  // --- UI store ---
  "ui.importing": "Importando",
  "ui.startingImport": "Iniciando importación…",
  "ui.importComplete": "¡Importación completada!",

  // --- Shortcut store and binding checks ---
  "shortcuts.loadFailed": "No se pudieron cargar los atajos de teclado: {error}",
  "shortcuts.updateFailed": "No se pudieron actualizar los atajos nativos del teclado: {error}",
  "shortcuts.reserved":
    "Este atajo está reservado para la edición de texto estándar o controles de ventana.",
  "shortcuts.conflict": "Ya está en uso por {label}. Primero borra o cambia ese atajo.",

  // --- Import helpers ---
  "import.selectScrivenerFolder": "Seleccionar carpeta de proyecto Scrivener (.scriv)",
  "import.scrivenerFolderOnly": "Por favor, selecciona una carpeta cuyo nombre termine en .scriv",
  "import.unknownType": "Tipo de importación desconocido: {type}",
  "import.failed": "La importación falló: {error}",

  // --- Import formats ---
  "importFormat.plottr": "Archivo de Plottr",
  "importFormat.markdown": "Archivo Markdown",
  "importFormat.ywriter": "Archivo de yWriter",
  "importFormat.longform": "Índice de Longform",
  "importFormat.longformVault": "Bóveda de Longform",
  "importFormat.novelwriter": "Proyecto novelWriter",
  "importFormat.scrivener": "Proyecto Scrivener",

  // --- Reference types ---
  "referenceType.characters": "Personajes",
  "referenceType.locations": "Ubicaciones",
  "referenceType.items": "Objetos",
  "referenceType.objectives": "Objetivos",
  "referenceType.organizations": "Organizaciones",
  "referenceType.timelines": "Cronologías",
  "referenceType.custom": "Notas",

  // --- Editorial review validation ---
  "editorial.unknownSource": "La revisión contiene una fuente de manuscrito desconocida.",
  "editorial.paragraphFormatting": "La revisión contiene un formato de párrafo no admitido.",
  "editorial.unsupportedLink": "La revisión contiene un enlace no admitido.",
  "editorial.outsidePassage":
    "Una anotación de feedback se refiere a un pasaje fuera de este manuscrito.",
  "editorial.mismatchedPassage": "Una anotación de feedback no coincide con su pasaje original.",
  "editorial.invalidAnchor": "Un comentario tiene un ancla de pasaje inválida.",
  "editorial.noSources": "La revisión no tiene fuentes de manuscrito.",
  "editorial.reorderSources":
    "Esta edición reordenaría las fuentes del manuscrito. Copia el pasaje como texto en su lugar.",
  "editorial.lostSource":
    "Un párrafo del manuscrito ha perdido su fuente. Reabre la revisión antes de continuar.",
  "editorial.noCurrentPassages":
    "No hay pasajes actuales del manuscrito en esta revisión. Restaura una escena antes de aplicar feedback.",
  "editorial.passageChanged":
    "Este pasaje cambió. Compara la prosa original, la actual y la sugerida, y luego selecciona dónde pertenece el cambio.",
  "editorial.overlap": "Estas sugerencias se superponen. Revísalas y acéptalas individualmente.",
  "editorial.selectPassage": "Selecciona un pasaje dentro del manuscrito.",

  // --- Prose save queue ---
  "proseSaves.draftsChanged":
    "Los borradores sin guardar cambiaron. Reintenta cargarlos antes de descartarlos.",
  "proseSaves.flushFailed":
    "No se pudo guardar la última prosa. {errors} Reintenta guardar o revisa los borradores sin guardar en Buscar y Reemplazar.",
  "proseFlush.message":
    "Guarda o recupera la prosa sin guardar antes de {action}. Revisa los borradores sin guardar en Buscar y Reemplazar.",
  "proseFlush.action.exporting": "exportar",
  "proseFlush.action.syncing": "sincronizar",

  // --- Editorial recovery and re-anchoring ---
  "editorialSaves.recoveredMismatch":
    "La revisión recuperada tiene un manuscrito diferente. Mantén esta ventana abierta y exporta una copia de recuperación.",
  "localEditorial.reanchor":
    "Selecciona un pasaje dentro de la prosa original de esta escena para volver a anclar su anotación guardada.",

  // --- Review suggestions ---
  "revisions.suggestionClosed": "La sugerencia ya no está abierta",
  "revisions.proseChanged":
    "La prosa cambió. Vuelve a anclar las sugerencias desactualizadas antes de aceptarlas.",
  "revisions.overlap": "Las sugerencias se superponen. Revísalas y acéptalas individualmente.",

  // --- Utility errors ---
  "referenceCopy.failed": "No se pudieron copiar las referencias. Inténtalo de nuevo.",
  "synopsisSaves.changed":
    "Los borradores de sinopsis cambiaron. Revisa los últimos cambios antes de descartarlos.",
  "exportPrototype.nameRequired": "Dale un nombre a este perfil antes de guardarlo.",
  "exportPrototype.startNumberInteger":
    "El número de inicio del capítulo debe ser un número entero.",
  "exportPrototype.unsupportedFormat":
    "Los perfiles de prototipo guardados tienen un formato no admitido.",
  "exportPrototype.invalidProfile": "Un perfil de prototipo guardado es inválido.",
  "exportPrototype.unsupportedOption": "Una opción de prototipo guardada no es compatible.",
  "exportPrototype.duplicateIds": "Los perfiles de prototipo guardados tienen IDs duplicados.",
  "exportPrototype.fontSize": "Tamaño de fuente",
  "exportPrototype.lineSpacing": "Espaciado de línea",
  "exportPrototype.paragraphSpacing": "Espaciado de párrafo",
  "exportPrototype.indent": "Sangría de primera línea",
  "exportPrototype.margin": "Márgenes",
  "exportPrototype.startNumber": "Número inicial de capítulo",
  "exportPrototype.rangeError": "{label} debe estar entre {min} y {max}.",

  // --- Command categories ---
  "commandCategory.File": "Archivo",
  "commandCategory.View": "Ver",
  "commandCategory.Edit": "Editar",
  "commandCategory.Help": "Ayuda",
  "commandCategory.Project": "Proyecto",
  "commandCategory.Editor": "Editor",

  // --- Command labels (mirror shortcutDefinitions.json) ---
  "command.editorial_open.label": "Abrir archivo de revisión o feedback",
  "command.editorial_project.label": "Revisión editorial y paquetes",
  "command.find.label": "Buscar en la escena",
  "command.find_replace.label": "Buscar y reemplazar",
  "command.find_project.label": "Buscar y reemplazar en el proyecto",
  "command.export.label": "Exportar proyecto",
  "command.close_project.label": "Cerrar proyecto",
  "command.quit.label": "Salir de Kindling",
  "command.import_plottr.label": "Importar Plottr (.pltr)",
  "command.import_markdown.label": "Importar Markdown (.md)",
  "command.import_longform.label": "Importar Longform",
  "command.import_ywriter.label": "Importar yWriter 7 (.yw7)",
  "command.import_scrivener.label": "Importar Scrivener 3 (.scriv)",
  "command.import_novelwriter.label": "Importar novelWriter",
  "command.sync.label": "Sincronizar desde origen",
  "command.toggle_sidebar.label": "Alternar barra lateral",
  "command.toggle_references.label": "Alternar panel de referencias",
  "command.toggle_discovery_notes.label": "Alternar notas de descubrimiento",
  "command.toggle_editor_mode.label": "Alternar vista de tiempo/página",
  "command.detect_references.label": "Detectar referencias en la escena",
  "command.detect_all_references.label": "Detectar referencias en todas las escenas",
  "command.about.label": "Acerca de kindling",
  "command.quick_start.label": "Guía de inicio rápido",
  "command.settings.label": "Ajustes",
  "command.new_project.label": "Nuevo proyecto",
  "command.command_palette.label": "Paleta de comandos",
  "command.send_feedback.label": "Enviar feedback",
  "command.editorial_comment.label": "Añadir comentario editorial",
  "command.bold.label": "Negrita",
  "command.italic.label": "Cursiva",
  "command.underline.label": "Subrayado",
  "command.strike.label": "Tachado",
  "command.code.label": "Monoespaciado",
  "command.blockquote.label": "Cita en bloque",
  "command.align_left.label": "Alinear a la izquierda",
  "command.align_center.label": "Alinear al centro",
  "command.align_right.label": "Alinear a la derecha",
  "command.align_justify.label": "Justificar",
};

export default catalog;
