import type { Catalog } from "../../types";

/** Settings dialog and its Appearance, Author, Keyboard and Project panes. */
const catalog: Catalog = {
  // --- Settings dialog: navigation and chrome ---
  "dialog.title": "Ajustes",
  "dialog.closeLabel": "Cerrar ajustes",
  "dialog.areasAria": "Áreas de ajustes",
  "dialog.groupPreferences": "Preferencias",
  "dialog.groupProjects": "Proyectos",
  "dialog.sectionManuscript": "Manuscrito",
  "dialog.sectionReferenceLibrary": "Biblioteca de Referencias",
  "dialog.areaAppearance": "Apariencia y Guía",
  "dialog.areaAuthor": "Autor y Contacto",
  "dialog.areaKeyboard": "Atajos de teclado",
  "dialog.areaDetails": "Detalles del Proyecto",
  "dialog.areaReferences": "Tipos de Referencia",
  "dialog.areaTags": "Etiquetas",
  "dialog.areaFields": "Campos Personalizados",
  "dialog.discardSettings": "¿Descartar los cambios de configuración sin guardar?",
  "dialog.discardProjectSwitch":
    "¿Descartar los cambios sin guardar para este proyecto antes de cambiar?",
  "dialog.keepEditing": "Seguir editando",
  "dialog.discard": "Descartar cambios",
  "dialog.loadingProjects": "Cargando proyectos…",
  "dialog.retryLoadProjects": "Reintentar cargar proyectos",
  "dialog.noProjects": "No hay proyectos disponibles.",
  "dialog.projectLabel": "Proyecto",
  "dialog.projectHelp":
    "Ajustes para este proyecto. Tu manuscrito abierto se mantendrá en su lugar.",
  "dialog.appliesToAll": "Se aplica a todos los proyectos.",
  "dialog.noProjectsHelp":
    "Aún no hay proyectos. Crea o importa un proyecto para configurar sus ajustes.",
  "dialog.loadProjectsFailed": "No se pudieron cargar los proyectos: {error}",

  // --- Appearance ---
  "appearance.lede":
    "Elige cómo se ve kindling y cuánta ayuda se muestra. Los cambios se aplican inmediatamente a todos los proyectos.",
  "appearance.title": "Apariencia",
  "appearance.themeAria": "Tema",
  "appearance.themeDark": "Oscuro",
  "appearance.themeLight": "Claro",
  "appearance.themeSystem": "Sistema",
  "appearance.systemHelp":
    "“Sistema” sigue la configuración de apariencia de tu sistema operativo.",
  "appearance.guidance": "Guía",
  "appearance.showTips": "Mostrar consejos de guía",
  "appearance.tipsHelp":
    "Aparece un consejo la primera vez que visitas la barra lateral, el panel de escena y las referencias. Cada consejo se muestra una vez.",
  "appearance.showTipsAgain": "Volver a mostrar consejos",
  "appearance.takeTour": "Hacer el recorrido",
  "appearance.turnOnTips": "Activa los consejos de guía para volver a mostrarlos.",
  "appearance.tipsReset": "Los consejos aparecerán de nuevo a medida que visites cada área.",

  // --- Author & contact ---
  "author.lede":
    "Tus datos de autor y contacto se aplican a las páginas de título de los manuscritos en todos los proyectos.",
  "author.loading": "Cargando detalles del autor…",
  "author.infoTitle": "Información del autor",
  "author.nameLabel": "Nombre del autor",
  "author.namePlaceholder": "Tu nombre legal",
  "author.nameHelp":
    "Utilizado en la información de contacto de las portadas. Los proyectos pueden sobreescribir esto con un seudónimo.",
  "author.contactTitle": "Información de contacto",
  "author.contactHelp":
    "Detalles opcionales para las páginas de título de los manuscritos. Usa cualquier formato que funcione para tu país.",
  "author.address1": "Línea de Dirección 1",
  "author.address1Placeholder": "Dirección o Apartado Postal",
  "author.address2": "Línea de Dirección 2",
  "author.address2Placeholder": "Ciudad, Estado/Provincia, Código Postal, País",
  "author.phone": "Teléfono",
  "author.phonePlaceholder": "+1 (555) 123-4567",
  "author.email": "Correo electrónico",
  "author.emailPlaceholder": "autor@email.com",
  "author.saved": "Detalles del autor guardados.",
  "author.save": "Guardar detalles del autor",
  "author.loadFailedTitle": "No se pudieron cargar los detalles del autor",
  "author.loadFailedHelp":
    "Si el archivo de ajustes está dañado, restablece los detalles del autor para restaurar el guardado y las exportaciones. Se conservará una copia de recuperación del archivo original.",
  "author.retry": "Reintentar cargar detalles del autor",
  "author.reset": "Restablecer detalles del autor",

  // --- Keyboard shortcuts ---
  "keyboard.lede":
    "Elige un atajo y presiona una nueva combinación de teclas. Los cambios se aplican inmediatamente a todos los proyectos. Escape cancela la grabación; Tabulador pasa al siguiente control.",
  "keyboard.systemHelp":
    "La edición de texto estándar y los controles del sistema (como copiar, pegar, deshacer, Tabulador y Escape) mantienen sus teclas habituales.",
  "keyboard.retryLoad": "Reintentar cargar atajos",
  "keyboard.filterLabel": "Filtrar comandos",
  "keyboard.resetDefaults": "Restablecer todo a los valores predeterminados",
  "keyboard.loading": "Cargando atajos…",
  "keyboard.shortcutFor": "Atajo para {label}",
  "keyboard.pressKeys": "Presiona teclas…",
  "keyboard.unassigned": "Sin asignar",
  "keyboard.clearFor": "Borrar atajo para {label}",
  "keyboard.clear": "Borrar",
  "keyboard.noMatches": "Ningún comando coincide.",
  "keyboard.saved": "Atajos de teclado guardados.",
  "keyboard.saveFailed": "No se pudieron guardar los atajos de teclado: {error}",
  "keyboard.invalidKey":
    "Usa Command en macOS o Ctrl en Windows/Linux, con una letra, número, signo de puntuación o F1–F24.",

  // --- Project details ---
  "project.ledePrefix": "Los cambios aquí solo se aplican a ",
  "project.ledeSuffix": ".",
  "project.tagChanges": "Los cambios en etiquetas se guardan a medida que los haces.",
  "project.fieldChanges":
    "Los cambios en campos personalizados se guardan a medida que los haces. Los campos se enumeran para los tipos de referencia guardados y activados.",
  "project.optional": "(opcional)",
  "project.penName": "Seudónimo",
  "project.penNamePlaceholder": "Déjalo en blanco para usar el nombre del autor",
  "project.penNameHelp":
    "Si se proporciona, se usará como firma en las páginas de título en lugar de tu nombre de autor.",
  "project.genre": "Género",
  "project.genrePlaceholder": "Ej., Ficción literaria, Ciencia ficción, Misterio",
  "project.genreHelp": "El género se mostrará en las páginas de título del manuscrito.",
  "project.description": "Descripción del Proyecto",
  "project.descriptionPlaceholder": "Breve resumen o notas sobre este proyecto",
  "project.dailyGoal": "Meta de escritura diaria",
  "project.loadDailyGoalFailed": "No se pudo cargar la meta diaria: {error}",
  "project.dailyGoalHelp":
    "Palabras netas añadidas por día en este proyecto. Pon 0 para desactivar la meta. Los cambios se aplican desde hoy; los días de racha anteriores mantienen sus metas originales.",
  "project.wordTarget": "Objetivo de palabras",
  "project.wordTargetPlaceholder": "Ej., 80000",
  "project.referencesHelp":
    "Elige qué tipos de referencia aparecen en el panel de Referencias de este proyecto. Deshabilitar un tipo mantiene sus entradas existentes.",
  "project.enableRefTypes": "Habilita tipos de referencia para configurar campos personalizados.",
  "project.saved": "Cambios del proyecto guardados.",
  "project.save": "Guardar cambios del proyecto",
  "project.wordTargetInvalid": "El objetivo de palabras debe ser un número entero no negativo",
  "project.dailyGoalInvalid": "La meta diaria debe ser un número entero entre 0 y 1.000.000",
  "project.disabledDraft":
    "Guarda o cancela el borrador del campo personalizado {label} antes de desactivar ese tipo de referencia.",
};

export default catalog;
