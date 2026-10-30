import type { Catalog } from "../../types";

/** Copy produced outside components: stores, utils, import formats and command labels. */
const catalog: Catalog = {
  // --- UI store ---
  "ui.importing": "正在导入",
  "ui.startingImport": "正在开始导入…",
  "ui.importComplete": "导入完成！",

  // --- Shortcut store and binding checks ---
  "shortcuts.loadFailed": "无法加载键盘快捷键：{error}",
  "shortcuts.updateFailed": "无法更新原生键盘快捷键：{error}",
  "shortcuts.reserved": "此快捷键已保留给标准文本编辑或窗口控件。",
  "shortcuts.conflict": "已被“{label}”占用。请先清除或更改那个快捷键。",

  // --- Import helpers ---
  "import.selectScrivenerFolder": "选择 Scrivener 项目文件夹（.scriv）",
  "import.scrivenerFolderOnly": "请选择名称以 .scriv 结尾的文件夹",
  "import.unknownType": "未知的导入类型：{type}",
  "import.failed": "导入失败：{error}",

  // --- Import formats ---
  "importFormat.plottr": "Plottr 文件",
  "importFormat.markdown": "Markdown 文件",
  "importFormat.ywriter": "yWriter 文件",
  "importFormat.longform": "Longform 索引",
  "importFormat.longformVault": "Longform 库",
  "importFormat.novelwriter": "novelWriter 项目",
  "importFormat.scrivener": "Scrivener 项目",

  // --- Reference types ---
  "referenceType.characters": "人物",
  "referenceType.locations": "地点",
  "referenceType.items": "物品",
  "referenceType.objectives": "目标",
  "referenceType.organizations": "组织",
  "referenceType.timelines": "时间线",
  "referenceType.custom": "笔记",

  // --- Editorial review validation ---
  "editorial.unknownSource": "审阅中包含未知的文稿来源。",
  "editorial.paragraphFormatting": "审阅中包含不支持的段落格式。",
  "editorial.unsupportedLink": "审阅中包含不支持的链接。",
  "editorial.outsidePassage": "有一条反馈批注指向本文稿之外的段落。",
  "editorial.mismatchedPassage": "有一条反馈批注与其原始段落不匹配。",
  "editorial.invalidAnchor": "批注的段落锚点无效。",
  "editorial.noSources": "审阅中没有文稿来源。",
  "editorial.reorderSources": "此编辑会重新排列文稿来源的顺序。请改为将段落复制为文本。",
  "editorial.lostSource": "有一段文稿已丢失来源。请重新打开审阅后再继续。",
  "editorial.noCurrentPassages": "此审阅中没有当前文稿段落。请先恢复一个场景再应用反馈。",
  "editorial.passageChanged":
    "此段落已更改。请比较原始、当前和建议的文稿，然后选择更改应放置的位置。",
  "editorial.overlap": "这些建议有重叠。请逐一审阅并接受。",
  "editorial.selectPassage": "请在文稿中选择一个段落。",

  // --- Prose save queue ---
  "proseSaves.draftsChanged": "未保存的草稿已更改。请重新加载后再放弃。",
  "proseSaves.flushFailed":
    "无法保存最新的文稿。{errors} 请重试保存，或在“查找和替换”中查看未保存的草稿。",
  "proseFlush.message":
    "请先保存或恢复未保存的文稿，再进行{action}。请在“查找和替换”中查看未保存的草稿。",
  "proseFlush.action.exporting": "导出",
  "proseFlush.action.syncing": "同步",

  // --- Editorial recovery and re-anchoring ---
  "editorialSaves.recoveredMismatch":
    "恢复的审阅对应的是另一份文稿。请保持此窗口打开，并导出一份恢复副本。",
  "localEditorial.reanchor": "请在此场景的原始文稿中选择一个段落，以重新锚定其已保存的批注。",

  // --- Review suggestions ---
  "revisions.suggestionClosed": "该建议已不再处于开放状态",
  "revisions.proseChanged": "文稿已更改。接受前请重新锚定过时的建议。",
  "revisions.overlap": "建议有重叠。请逐一审阅并接受。",

  // --- Command categories ---
  "commandCategory.File": "文件",
  "commandCategory.View": "视图",
  "commandCategory.Edit": "编辑",
  "commandCategory.Help": "帮助",
  "commandCategory.Project": "项目",
  "commandCategory.Editor": "编辑器",

  // --- Command labels (mirror shortcutDefinitions.json) ---
  "command.editorial_open.label": "打开审阅或反馈文件",
  "command.editorial_project.label": "审阅与反馈包",
  "command.find.label": "在场景中查找",
  "command.find_replace.label": "查找并替换",
  "command.find_project.label": "在项目中查找并替换",
  "command.export.label": "导出项目",
  "command.close_project.label": "关闭项目",
  "command.quit.label": "退出 Kindling",
  "command.import_plottr.label": "导入 Plottr (.pltr)",
  "command.import_markdown.label": "导入 Markdown (.md)",
  "command.import_longform.label": "导入 Longform",
  "command.import_ywriter.label": "导入 yWriter 7 (.yw7)",
  "command.import_scrivener.label": "导入 Scrivener 3 (.scriv)",
  "command.import_novelwriter.label": "导入 novelWriter",
  "command.sync.label": "从源文件同步",
  "command.toggle_sidebar.label": "显示/隐藏侧边栏",
  "command.toggle_references.label": "显示/隐藏引用面板",
  "command.toggle_discovery_notes.label": "显示/隐藏发现笔记",
  "command.toggle_editor_mode.label": "切换节拍/页面视图",
  "command.detect_references.label": "检测场景中的引用",
  "command.detect_all_references.label": "检测所有场景中的引用",
  "command.about.label": "关于 kindling",
  "command.quick_start.label": "快速入门指南",
  "command.settings.label": "设置",
  "command.new_project.label": "新建项目",
  "command.command_palette.label": "命令面板",
  "command.send_feedback.label": "发送反馈",
  "command.editorial_comment.label": "添加审阅批注",
  "command.bold.label": "粗体",
  "command.italic.label": "斜体",
  "command.underline.label": "下划线",
  "command.strike.label": "删除线",
  "command.code.label": "等宽字体",
  "command.blockquote.label": "引用块",
  "command.align_left.label": "左对齐",
  "command.align_center.label": "居中对齐",
  "command.align_right.label": "右对齐",
  "command.align_justify.label": "两端对齐",
};

export default catalog;
