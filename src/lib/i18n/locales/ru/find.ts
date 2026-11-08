import type { Catalog } from "../../types";

/** Find and Replace dialog. */
const catalog: Catalog = {
  "findReplaceDialog.title": "Найти и заменить",
  "findReplaceDialog.find": "Найти",
  "findReplaceDialog.previousMatch": "Предыдущее совпадение (Shift+Enter)",
  "findReplaceDialog.previous": "Предыдущее",
  "findReplaceDialog.nextMatch": "Следующее совпадение (Enter)",
  "findReplaceDialog.next": "Следующее",
  "findReplaceDialog.loadingProse": "Загрузка прозы…",
  "findReplaceDialog.enterQuery": "Введите текст для поиска.",
  "findReplaceDialog.searchIn": "Искать в",
  "findReplaceDialog.currentScene": "Текущая сцена",
  "findReplaceDialog.entireProject": "Весь проект",
  "findReplaceDialog.matchCase": "С учётом регистра",
  "findReplaceDialog.wholeWords": "Целые слова",
  "findReplaceDialog.replace": "Заменить",
  "findReplaceDialog.replaceWith": "Заменить на",
  "findReplaceDialog.scopeHelp":
    "Поиск выполняется в видимой прозе в фиксированных сценах. Гибкие, неопределённые и архивные сцены исключены; заблокированные сцены доступны для поиска, но не для замены.",
  "findReplaceDialog.position": "{current} из {total}",
  "findReplaceDialog.locked": " · Заблокировано",
  "findReplaceDialog.unsavedDraft": " · Несохранённый черновик",
  "findReplaceDialog.openScene": "Открыть сцену",
  "findReplaceDialog.noMoreMatches":
    "Совпадений впереди больше нет. Используйте «Следующее» или «Предыдущее» для продолжения.",
  "findReplaceDialog.noMatches": "Совпадений не найдено.",
  "findReplaceDialog.confirmReplaceAll": "Подтвердить замену всех",
  "findReplaceDialog.retryLoading": "Повторить загрузку",
  "findReplaceDialog.retrySavingDrafts": "Повторить сохранение черновиков",
  "findReplaceDialog.draftsHelp":
    "Несохранённые черновики сохраняются для этого сеанса, в том числе после закрытия проекта. Заблокированные или отсутствующие документы и нераспознанные ошибки сохранения не повторяются автоматически и не блокируют другие сцены. Вы можете повторить попытку после устранения ошибки или скопировать черновики перед их удалением. Совпадения в документах с несохранёнными черновиками нельзя заменить.",
  "findReplaceDialog.unsavedScene": "Несохранённая сцена {index}",
  "findReplaceDialog.unsavedBeat": "Несохранённый бит {index}",
  "findReplaceDialog.draftText": "Текст черновика (выделите для копирования)",
  "findReplaceDialog.discardQuestion":
    "Удалить эти несохранённые черновики? Их изменения будут потеряны. Сначала скопируйте нужный текст.",
  "findReplaceDialog.confirmDiscardDrafts": "Подтвердить удаление черновиков",
  "findReplaceDialog.keepDrafts": "Сохранить черновики",
  "findReplaceDialog.discardDrafts": "Удалить несохранённые черновики…",
  "findReplaceDialog.undoReplacement": "Отменить замену",
  "findReplaceDialog.replaceAll": "Заменить все",
  "findReplaceDialog.replaceMatch": "Заменить совпадение",
  "findReplaceDialog.replacementUndone": "Замена отменена.",
  "findReplaceDialog.replacementSaved": "Замена сохранена.",
  "findReplaceDialog.noChangesNeeded": "Изменений не требуется.",
  "findReplaceDialog.confirmReplaceProject.one": "Заменить {count} совпадение во всём проекте?",
  "findReplaceDialog.confirmReplaceProject.few": "Заменить {count} совпадения во всём проекте?",
  "findReplaceDialog.confirmReplaceProject.many": "Заменить {count} совпадений во всём проекте?",
  "findReplaceDialog.confirmReplaceProject.other": "Заменить {count} совпадения во всём проекте?",
  "findReplaceDialog.confirmReplaceScene.one": "Заменить {count} совпадение в текущей сцене?",
  "findReplaceDialog.confirmReplaceScene.few": "Заменить {count} совпадения в текущей сцене?",
  "findReplaceDialog.confirmReplaceScene.many": "Заменить {count} совпадений в текущей сцене?",
  "findReplaceDialog.confirmReplaceScene.other": "Заменить {count} совпадения в текущей сцене?",
  "findReplaceDialog.confirmSkipped.one":
    "{count} заблокированное или несохранённое совпадение будет пропущено.",
  "findReplaceDialog.confirmSkipped.few":
    "{count} заблокированных или несохранённых совпадения будут пропущены.",
  "findReplaceDialog.confirmSkipped.many":
    "{count} заблокированных или несохранённых совпадений будут пропущены.",
  "findReplaceDialog.confirmSkipped.other":
    "{count} заблокированных или несохранённых совпадения будут пропущены.",
};

export default catalog;
