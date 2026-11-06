# Interface glossary

Core writing terms and how each locale translates them. Use these renderings everywhere in a
locale's catalog so the same idea always reads the same way. When adding a language, add a column
and fill it before translating the catalogs.

The zh-CN column is taken from the shipped catalogs in `src/lib/i18n/locales/zh-CN/`.

| Term             | Meaning in kindling                                                  | en               | zh-CN |
| ---------------- | -------------------------------------------------------------------- | ---------------- | ----- |
| beat             | A single story point inside a scene, imported from the outline       | beat             | 节拍  |
| scene            | The unit of prose; holds beats, a synopsis and the draft text        | scene            | 场景  |
| chapter          | A group of scenes (called a sequence in screenplay projects)         | chapter          | 章    |
| part             | A group of chapters (called an act in screenplay projects)           | part             | 部    |
| act              | Screenplay name for a part                                           | act              | 幕    |
| sequence         | Screenplay name for a chapter                                        | sequence         | 序列  |
| outline          | The plan imported from Plottr, Scrivener and the like                | outline          | 大纲  |
| synopsis         | A short summary of a scene or chapter                                | synopsis         | 梗概  |
| manuscript       | The whole draft as a reader or editor sees it                        | manuscript       | 稿件  |
| prose            | The written text of a scene                                          | prose            | 正文  |
| reference        | A character, location or other note linked to scenes                 | reference        | 引用  |
| snapshot         | A saved copy of the whole project that can be restored               | snapshot         | 快照  |
| revision         | A saved version of a scene's prose                                   | revision         | 修订  |
| draft            | A scene status, and the automatic saved versions in revision history | draft            | 草稿  |
| editorial review | Reading the manuscript and leaving suggestions and comments          | editorial review | 审阅  |
| feedback         | Suggestions and comments from a review, shared as a file             | feedback         | 反馈  |

## Known inconsistencies

These shipped zh-CN strings disagree with the table above and should be aligned in a later change:

- **manuscript**: `settings.dialog.sectionManuscript` uses 文稿; the editorial workspace uses 稿件.
- **sequence**: `editor.scenePanel.sequence` uses 场次; the sidebar and counts use 序列.
- **revision**: `editor.scenePanel.revisions` reads 修订历史 (revision history), which is fine as a
  panel title; counted revisions should use 修订.
