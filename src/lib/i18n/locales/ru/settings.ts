import type { Catalog } from "../../types";

/** Settings dialog and its Appearance, Author, Keyboard and Project panes. */
const catalog: Catalog = {
  // --- Settings dialog: navigation and chrome ---
  "dialog.title": "Настройки",
  "dialog.closeLabel": "Закрыть настройки",
  "dialog.areasAria": "Разделы настроек",
  "dialog.groupPreferences": "Предпочтения",
  "dialog.groupProjects": "Проекты",
  "dialog.sectionManuscript": "Рукопись",
  "dialog.sectionReferenceLibrary": "Библиотека ссылок",
  "dialog.areaAppearance": "Внешний вид и подсказки",
  "dialog.areaAuthor": "Автор и контакты",
  "dialog.areaKeyboard": "Сочетания клавиш",
  "dialog.areaDetails": "Сведения о проекте",
  "dialog.areaReferences": "Типы ссылок",
  "dialog.areaTags": "Теги",
  "dialog.areaFields": "Пользовательские поля",
  "dialog.discardSettings": "Отменить несохранённые изменения настроек?",
  "dialog.discardProjectSwitch":
    "Отменить несохранённые изменения этого проекта перед переключением?",
  "dialog.keepEditing": "Продолжить редактирование",
  "dialog.discard": "Отменить изменения",
  "dialog.loadingProjects": "Загрузка проектов…",
  "dialog.retryLoadProjects": "Повторить загрузку проектов",
  "dialog.noProjects": "Нет доступных проектов.",
  "dialog.projectLabel": "Проект",
  "dialog.projectHelp": "Настройки для этого проекта. Открытая рукопись остаётся на месте.",
  "dialog.appliesToAll": "Применяется ко всем проектам.",
  "dialog.noProjectsHelp":
    "Проектов пока нет. Создайте или импортируйте проект для настройки его параметров.",
  "dialog.loadProjectsFailed": "Не удалось загрузить проекты: {error}",

  // --- Appearance ---
  "appearance.lede":
    "Выберите внешний вид kindling и количество отображаемых подсказок. Изменения немедленно применяются ко всем проектам.",
  "appearance.title": "Внешний вид",
  "appearance.themeAria": "Тема",
  "appearance.themeDark": "Тёмная",
  "appearance.themeLight": "Светлая",
  "appearance.themeSystem": "Системная",
  "appearance.systemHelp":
    "«Системная» соответствует настройке оформления вашей операционной системы.",
  "appearance.guidance": "Подсказки",
  "appearance.showTips": "Показывать подсказки",
  "appearance.tipsHelp":
    "Подсказка появляется при первом посещении боковой панели, панели сцены и ссылок. Каждая подсказка показывается один раз.",
  "appearance.showTipsAgain": "Показать подсказки снова",
  "appearance.takeTour": "Пройти тур",
  "appearance.turnOnTips": "Включите подсказки, чтобы показывать их снова.",
  "appearance.tipsReset": "Подсказки появятся снова при посещении каждого раздела.",

  // --- Author & contact ---
  "author.lede":
    "Данные об авторе и контактная информация применяются к титульным страницам рукописей во всех проектах.",
  "author.loading": "Загрузка данных об авторе…",
  "author.infoTitle": "Информация об авторе",
  "author.nameLabel": "Имя автора",
  "author.namePlaceholder": "Ваше настоящее имя",
  "author.nameHelp":
    "Используется в контактной информации на титульных страницах. Проекты могут переопределить это псевдонимом.",
  "author.contactTitle": "Контактная информация",
  "author.contactHelp":
    "Необязательные данные для титульных страниц рукописей. Используйте любой формат, подходящий для вашей страны.",
  "author.address1": "Адрес, строка 1",
  "author.address1Placeholder": "Улица или абонентский ящик",
  "author.address2": "Адрес, строка 2",
  "author.address2Placeholder": "Город, регион, почтовый индекс, страна",
  "author.phone": "Телефон",
  "author.phonePlaceholder": "+7 (999) 123-45-67",
  "author.email": "Электронная почта",
  "author.emailPlaceholder": "автор@email.com",
  "author.saved": "Данные об авторе сохранены.",
  "author.save": "Сохранить данные об авторе",
  "author.loadFailedTitle": "Не удалось загрузить данные об авторе",
  "author.loadFailedHelp":
    "Если файл настроек повреждён, сбросьте данные об авторе для восстановления сохранения и экспорта. Восстановительная копия исходного файла будет сохранена.",
  "author.retry": "Повторить загрузку данных об авторе",
  "author.reset": "Сбросить данные об авторе",

  // --- Keyboard shortcuts ---
  "keyboard.lede":
    "Выберите сочетание, затем нажмите новую комбинацию клавиш. Изменения немедленно применяются ко всем проектам. Escape отменяет запись; Tab переходит к следующему элементу.",
  "keyboard.systemHelp":
    "Стандартное редактирование текста и системные элементы управления (например, копирование, вставка, отмена, Tab и Escape) сохраняют свои обычные клавиши.",
  "keyboard.retryLoad": "Повторить загрузку сочетаний клавиш",
  "keyboard.filterLabel": "Фильтр команд",
  "keyboard.resetDefaults": "Сбросить все по умолчанию",
  "keyboard.loading": "Загрузка сочетаний клавиш…",
  "keyboard.shortcutFor": "Сочетание для {label}",
  "keyboard.pressKeys": "Нажмите клавиши…",
  "keyboard.unassigned": "Не назначено",
  "keyboard.clearFor": "Очистить сочетание для {label}",
  "keyboard.clear": "Очистить",
  "keyboard.noMatches": "Нет подходящих команд.",
  "keyboard.saved": "Сочетания клавиш сохранены.",
  "keyboard.saveFailed": "Не удалось сохранить сочетания клавиш: {error}",
  "keyboard.invalidKey":
    "Используйте Command на macOS или Ctrl в Windows/Linux вместе с буквой, цифрой, знаком пунктуации или F1–F24.",

  // --- Project details ---
  "project.ledePrefix": "Изменения здесь применяются только к ",
  "project.ledeSuffix": ".",
  "project.tagChanges": "Изменения тегов сохраняются по мере внесения.",
  "project.fieldChanges":
    "Изменения пользовательских полей сохраняются по мере внесения. Поля перечислены для сохранённых, включённых типов ссылок.",
  "project.optional": "(необязательно)",
  "project.penName": "Псевдоним",
  "project.penNamePlaceholder": "Оставьте пустым, чтобы использовать имя автора",
  "project.penNameHelp":
    "Если указан, будет использоваться в качестве подписи на титульных страницах вместо имени автора.",
  "project.genre": "Жанр",
  "project.genrePlaceholder": "напр., Литературная проза, Научная фантастика, Детектив",
  "project.genreHelp": "Жанр будет отображаться на титульных страницах рукописей.",
  "project.description": "Описание проекта",
  "project.descriptionPlaceholder": "Краткое резюме или заметки об этом проекте",
  "project.dailyGoal": "Ежедневная цель написания",
  "project.loadDailyGoalFailed": "Не удалось загрузить ежедневную цель: {error}",
  "project.dailyGoalHelp":
    "Нетто слов, добавленных в день в этом проекте. Установите 0, чтобы отключить цель. Изменения применяются сегодня; более ранние дни серии сохраняют исходные цели.",
  "project.wordTarget": "Целевой объём слов",
  "project.wordTargetPlaceholder": "напр., 80000",
  "project.referencesHelp":
    "Выберите, какие типы ссылок отображаются в панели ссылок этого проекта. Отключение типа сохраняет существующие записи.",
  "project.enableRefTypes": "Включите типы ссылок для настройки пользовательских полей.",
  "project.saved": "Изменения проекта сохранены.",
  "project.save": "Сохранить изменения проекта",
  "project.wordTargetInvalid": "Целевой объём слов должен быть неотрицательным целым числом",
  "project.dailyGoalInvalid": "Ежедневная цель должна быть целым числом от 0 до 1 000 000",
  "project.disabledDraft":
    "Сохраните или отмените черновик пользовательского поля {label} перед отключением этого типа ссылок.",
};

export default catalog;
