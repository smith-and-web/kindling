import type { Catalog } from "../../types";

/** Copy produced outside components: stores, utils, import formats and command labels. */
const catalog: Catalog = {
  // --- UI store ---
  "ui.importing": "Импорт",
  "ui.startingImport": "Начало импорта…",
  "ui.importComplete": "Импорт завершён!",

  // --- Shortcut store and binding checks ---
  "shortcuts.loadFailed": "Не удалось загрузить сочетания клавиш: {error}",
  "shortcuts.updateFailed": "Не удалось обновить нативные сочетания клавиш: {error}",
  "shortcuts.reserved":
    "Это сочетание зарезервировано для стандартного редактирования текста или управления окнами.",
  "shortcuts.conflict":
    "Уже используется для {label}. Сначала очистите или измените это сочетание.",

  // --- Import helpers ---
  "import.selectScrivenerFolder": "Выберите папку проекта Scrivener (.scriv)",
  "import.scrivenerFolderOnly": "Пожалуйста, выберите папку, имя которой заканчивается на .scriv",
  "import.unknownType": "Неизвестный тип импорта: {type}",
  "import.failed": "Импорт не удался: {error}",

  // --- Import formats ---
  "importFormat.plottr": "Файл Plottr",
  "importFormat.markdown": "Файл Markdown",
  "importFormat.ywriter": "Файл yWriter",
  "importFormat.longform": "Индекс Longform",
  "importFormat.longformVault": "Хранилище Longform",
  "importFormat.novelwriter": "Проект novelWriter",
  "importFormat.scrivener": "Проект Scrivener",

  // --- Reference types ---
  "referenceType.characters": "Персонажи",
  "referenceType.locations": "Места",
  "referenceType.items": "Предметы",
  "referenceType.objectives": "Цели",
  "referenceType.organizations": "Организации",
  "referenceType.timelines": "Хронологии",
  "referenceType.custom": "Заметки",

  // --- Editorial review validation ---
  "editorial.unknownSource": "Рецензия содержит неизвестный источник рукописи.",
  "editorial.paragraphFormatting": "Рецензия содержит неподдерживаемое форматирование абзацев.",
  "editorial.unsupportedLink": "Рецензия содержит неподдерживаемую ссылку.",
  "editorial.outsidePassage": "Аннотация отзыва ссылается на фрагмент за пределами этой рукописи.",
  "editorial.mismatchedPassage": "Аннотация отзыва не соответствует исходному фрагменту.",
  "editorial.invalidAnchor": "Комментарий имеет недопустимый якорь фрагмента.",
  "editorial.noSources": "Рецензия не имеет источников рукописи.",
  "editorial.reorderSources":
    "Это изменение изменит порядок источников рукописи. Скопируйте фрагмент как текст.",
  "editorial.lostSource":
    "Абзац рукописи потерял источник. Повторно откройте рецензию перед продолжением.",
  "editorial.noCurrentPassages":
    "В этой рецензии нет текущих фрагментов рукописи. Восстановите сцену перед применением отзыва.",
  "editorial.passageChanged":
    "Этот фрагмент изменился. Сравните исходную, текущую и предложенную прозу, затем выберите, куда относится изменение.",
  "editorial.overlap": "Эти предложения перекрываются. Просмотрите и примите их по отдельности.",
  "editorial.selectPassage": "Выберите фрагмент в рукописи.",

  // --- Prose save queue ---
  "proseSaves.draftsChanged":
    "Несохранённые черновики изменились. Повторите загрузку перед удалением.",
  "proseSaves.flushFailed":
    "Не удалось сохранить последнюю прозу. {errors} Повторите сохранение или просмотрите несохранённые черновики в «Поиске и замене».",
  "proseFlush.message":
    "Сохраните или восстановите несохранённую прозу перед {action}. Просмотрите несохранённые черновики в «Поиске и замене».",
  "proseFlush.action.exporting": "экспортом",
  "proseFlush.action.syncing": "синхронизацией",

  // --- Editorial recovery and re-anchoring ---
  "editorialSaves.recoveredMismatch":
    "Восстановленная рецензия содержит другую рукопись. Не закрывайте это окно и экспортируйте копию восстановления.",
  "localEditorial.reanchor":
    "Выберите фрагмент в исходной прозе этой сцены, чтобы переякорить сохранённую аннотацию.",

  // --- Review suggestions ---
  "revisions.suggestionClosed": "Предложение больше не открыто",
  "revisions.proseChanged":
    "Проза изменилась. Переякорьте устаревшие предложения перед их принятием.",
  "revisions.overlap": "Предложения перекрываются. Просмотрите и примите их по отдельности.",

  // --- Utility errors ---
  "referenceCopy.failed": "Не удалось скопировать ссылки. Повторите попытку.",
  "synopsisSaves.changed":
    "Черновики краткого содержания изменились. Просмотрите последние изменения перед удалением.",
  "exportPrototype.nameRequired": "Дайте этому профилю название перед сохранением.",
  "exportPrototype.startNumberInteger": "Начальный номер главы должен быть целым числом.",
  "exportPrototype.unsupportedFormat":
    "Сохранённые прототипы профилей имеют неподдерживаемый формат.",
  "exportPrototype.invalidProfile": "Сохранённый прототип профиля недействителен.",
  "exportPrototype.unsupportedOption": "Сохранённый параметр прототипа не поддерживается.",
  "exportPrototype.duplicateIds":
    "Сохранённые прототипы профилей имеют дублирующиеся идентификаторы.",
  "exportPrototype.fontSize": "Размер шрифта",
  "exportPrototype.lineSpacing": "Межстрочный интервал",
  "exportPrototype.paragraphSpacing": "Межабзацный интервал",
  "exportPrototype.indent": "Отступ первой строки",
  "exportPrototype.margin": "Поля",
  "exportPrototype.startNumber": "Начальный номер главы",
  "exportPrototype.rangeError": "{label} должен быть между {min} и {max}.",

  // --- Command categories ---
  "commandCategory.File": "Файл",
  "commandCategory.View": "Вид",
  "commandCategory.Edit": "Правка",
  "commandCategory.Help": "Справка",
  "commandCategory.Project": "Проект",
  "commandCategory.Editor": "Редактор",

  // --- Command labels (mirror shortcutDefinitions.json) ---
  "command.editorial_open.label": "Открыть файл рецензии или отзыва",
  "command.editorial_project.label": "Редакционная рецензия и пакеты",
  "command.find.label": "Найти в сцене",
  "command.find_replace.label": "Найти и заменить",
  "command.find_project.label": "Найти и заменить в проекте",
  "command.export.label": "Экспортировать проект",
  "command.close_project.label": "Закрыть проект",
  "command.quit.label": "Выйти из Kindling",
  "command.import_plottr.label": "Импортировать Plottr (.pltr)",
  "command.import_markdown.label": "Импортировать Markdown (.md)",
  "command.import_longform.label": "Импортировать Longform",
  "command.import_ywriter.label": "Импортировать yWriter 7 (.yw7)",
  "command.import_scrivener.label": "Импортировать Scrivener 3 (.scriv)",
  "command.import_novelwriter.label": "Импортировать novelWriter",
  "command.sync.label": "Синхронизировать из источника",
  "command.toggle_sidebar.label": "Показать/скрыть боковую панель",
  "command.toggle_references.label": "Показать/скрыть панель ссылок",
  "command.toggle_discovery_notes.label": "Показать/скрыть заметки открытий",
  "command.toggle_editor_mode.label": "Переключить вид битов/страницы",
  "command.detect_references.label": "Обнаружить ссылки в сцене",
  "command.detect_all_references.label": "Обнаружить ссылки во всех сценах",
  "command.about.label": "О программе kindling",
  "command.quick_start.label": "Руководство быстрого старта",
  "command.settings.label": "Настройки",
  "command.new_project.label": "Новый проект",
  "command.command_palette.label": "Палитра команд",
  "command.send_feedback.label": "Отправить отзыв",
  "command.editorial_comment.label": "Добавить редакционный комментарий",
  "command.bold.label": "Полужирный",
  "command.italic.label": "Курсив",
  "command.underline.label": "Подчёркнутый",
  "command.strike.label": "Зачёркнутый",
  "command.code.label": "Моноширинный",
  "command.blockquote.label": "Блочная цитата",
  "command.align_left.label": "По левому краю",
  "command.align_center.label": "По центру",
  "command.align_right.label": "По правому краю",
  "command.align_justify.label": "По ширине",
};

export default catalog;
