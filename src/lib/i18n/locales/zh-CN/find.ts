import type { Catalog } from "../../types";

/** Find and Replace dialog. */
const catalog: Catalog = {
  "findReplaceDialog.title": "查找和替换",
  "findReplaceDialog.find": "查找",
  "findReplaceDialog.previousMatch": "上一处匹配（Shift+Enter）",
  "findReplaceDialog.previous": "上一处",
  "findReplaceDialog.nextMatch": "下一处匹配（Enter）",
  "findReplaceDialog.next": "下一处",
  "findReplaceDialog.loadingProse": "正在加载正文…",
  "findReplaceDialog.enterQuery": "输入要查找的文字。",
  "findReplaceDialog.searchIn": "查找范围",
  "findReplaceDialog.currentScene": "当前场景",
  "findReplaceDialog.entireProject": "整个项目",
  "findReplaceDialog.matchCase": "区分大小写",
  "findReplaceDialog.wholeWords": "全字匹配",
  "findReplaceDialog.replace": "替换",
  "findReplaceDialog.replaceWith": "替换为",
  "findReplaceDialog.scopeHelp":
    "只在“固定”场景的可见正文中查找。“灵活”“未定义”和已归档的场景不在范围内；锁定的场景可以查找但不能替换。",
  "findReplaceDialog.position": "第 {current} / {total} 处",
  "findReplaceDialog.locked": " · 已锁定",
  "findReplaceDialog.unsavedDraft": " · 未保存的草稿",
  "findReplaceDialog.openScene": "打开场景",
  "findReplaceDialog.noMoreMatches": "后面没有更多匹配。可用“下一处”或“上一处”继续查找。",
  "findReplaceDialog.noMatches": "没有找到匹配项。",
  "findReplaceDialog.confirmReplace":
    "在{scope}中替换 {count} 处匹配？另有 {skipped} 处匹配因锁定或未保存会被跳过。",
  "findReplaceDialog.scopeEntireProject": "整个项目",
  "findReplaceDialog.scopeCurrentScene": "当前场景",
  "findReplaceDialog.confirmReplaceAll": "确认全部替换",
  "findReplaceDialog.retryLoading": "重试加载",
  "findReplaceDialog.retrySavingDrafts": "重试保存草稿",
  "findReplaceDialog.draftsHelp":
    "未保存的草稿会在本次会话中保留，关闭项目后也不会丢失。锁定或缺失的文档、无法识别的保存错误不会自动重试，也不会阻塞其他场景。你可以在解决保存问题后重试，或先复制这些草稿再丢弃它们。含未保存草稿的文档无法替换。",
  "findReplaceDialog.unsavedScene": "未保存的场景 {index}",
  "findReplaceDialog.unsavedBeat": "未保存的节拍 {index}",
  "findReplaceDialog.draftText": "草稿文本（选中即可复制）",
  "findReplaceDialog.discardQuestion":
    "丢弃这些未保存的草稿？其中的修改将会丢失。请先复制需要保留的文字。",
  "findReplaceDialog.confirmDiscardDrafts": "确认丢弃草稿",
  "findReplaceDialog.keepDrafts": "保留草稿",
  "findReplaceDialog.discardDrafts": "丢弃未保存的草稿…",
  "findReplaceDialog.undoReplacement": "撤销替换",
  "findReplaceDialog.replaceAll": "全部替换",
  "findReplaceDialog.replaceMatch": "替换当前项",
  "findReplaceDialog.replacementUndone": "已撤销替换。",
  "findReplaceDialog.replacementSaved": "替换已保存。",
  "findReplaceDialog.noChangesNeeded": "无需修改。",
};

export default catalog;
