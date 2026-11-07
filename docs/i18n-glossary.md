# Interface glossary

Core writing terms and how each locale translates them. Use these renderings everywhere in a
locale's catalog so the same idea always reads the same way. When adding a language, add a column
and fill it before translating the catalogs.

The zh-CN column is taken from the shipped catalogs in `src/lib/i18n/locales/zh-CN/`.
The pt-BR column is taken from the shipped catalogs in `src/lib/i18n/locales/pt-BR/`.
The ru column is taken from the shipped catalogs in `src/lib/i18n/locales/ru/`.
The es column follows the terms in `src/lib/i18n/locales/es/`; "beat" (tiempo) and "feedback"
(left in English) are still open for the reviewer.
The de column follows the terms in `src/lib/i18n/locales/de/`; "Beat" is preferred over
"Handlungsschritt" following German screenwriting usage, pending native review (AC-4).
The fr column follows the terms in `src/lib/i18n/locales/fr/`; "temps fort" is preferred over
the borrowed "beat" following French storytelling usage, and "fiche" is used for "reference"
following French writing-tool conventions; both are pending native review (AC-4).

| Term             | Meaning in kindling                                                  | en               | zh-CN | pt-BR             | es                 | ru                    | de              | fr                    |
| ---------------- | -------------------------------------------------------------------- | ---------------- | ----- | ----------------- | ------------------ | --------------------- | --------------- | --------------------- |
| beat             | A single story point inside a scene, imported from the outline       | beat             | 节拍  | beat              | tiempo             | бит                   | Beat            | temps fort            |
| scene            | The unit of prose; holds beats, a synopsis and the draft text        | scene            | 场景  | cena              | escena             | сцена                 | Szene           | scène                 |
| chapter          | A group of scenes (called a sequence in screenplay projects)         | chapter          | 章    | capítulo          | capítulo           | глава                 | Kapitel         | chapitre              |
| part             | A group of chapters (called an act in screenplay projects)           | part             | 部    | parte             | parte              | часть                 | Teil            | partie                |
| act              | Screenplay name for a part                                           | act              | 幕    | ato               | acto               | акт                   | Akt             | acte                  |
| sequence         | Screenplay name for a chapter                                        | sequence         | 序列  | sequência         | secuencia          | секвенция             | Sequenz         | séquence              |
| outline          | The plan imported from Plottr, Scrivener and the like                | outline          | 大纲  | estrutura         | esquema            | структура             | Gliederung      | plan                  |
| synopsis         | A short summary of a scene or chapter                                | synopsis         | 梗概  | sinopse           | sinopsis           | краткое содержание    | Zusammenfassung | synopsis              |
| manuscript       | The whole draft as a reader or editor sees it                        | manuscript       | 稿件  | manuscrito        | manuscrito         | рукопись              | Manuskript      | manuscrit             |
| prose            | The written text of a scene                                          | prose            | 正文  | prosa             | prosa              | проза                 | Text            | texte                 |
| reference        | A character, location or other note linked to scenes                 | reference        | 引用  | referência        | referencia         | ссылка                | Referenz        | fiche                 |
| snapshot         | A saved copy of the whole project that can be restored               | snapshot         | 快照  | instantâneo       | instantánea        | снимок                | Schnappschuss   | instantané            |
| revision         | A saved version of a scene's prose                                   | revision         | 修订  | revisão           | revisión           | версия                | Version         | version               |
| draft            | A scene status, and the automatic saved versions in revision history | draft            | 草稿  | rascunho          | borrador           | черновик              | Entwurf         | brouillon             |
| editorial review | Reading the manuscript and leaving suggestions and comments          | editorial review | 审阅  | revisão editorial | revisión editorial | редакционная рецензия | Lektorat        | relecture éditoriale  |
| feedback         | Suggestions and comments from a review, shared as a file             | feedback         | 反馈  | feedback          | feedback           | обратная связь        | Rückmeldung     | retours               |

## Known inconsistencies

These shipped zh-CN strings disagree with the table above and should be aligned in a later change:

- **manuscript**: `settings.dialog.sectionManuscript` uses 文稿; the editorial workspace uses 稿件.
- **sequence**: `editor.scenePanel.sequence` uses 场次; the sidebar and counts use 序列.
- **revision**: `editor.scenePanel.revisions` reads 修订历史 (revision history), which is fine as a
  panel title; counted revisions should use 修订.
