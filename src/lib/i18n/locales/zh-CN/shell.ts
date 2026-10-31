import type { Catalog } from "../../types";

/** App shell: quit flow, save failures, toasts and dialog chrome. */
const catalog: Catalog = {
  quitReviewTitle: "不保存审阅修改就退出？",
  quitWritingTitle: "不保存写作修改就退出？",
  quitSynopsisTitle: "不保存梗概修改就退出？",
  quitReviewMessage:
    "你的审阅修改无法保存。可以继续编辑重试，或导出一份恢复副本。退出并放弃会丢弃所有未保存的审阅、正文和梗概修改。",
  quitWritingMessage:
    "部分正文或梗概修改无法保存。可以退出并放弃这些未保存的写作修改，或继续编辑重试保存。",
  quitSynopsisMessage:
    "部分梗概修改无法保存。可以退出并放弃这些未保存的梗概修改，或继续编辑重试保存。",
  quitAndDiscard: "退出并放弃",
  keepEditing: "继续编辑",
  couldNotQuit: "无法退出：{error}",
  sceneUnavailable: "该场景已不存在。",
  proseUnsaved: "正文修改无法保存。请在“查找和替换”中检查未保存的草稿。",
  synopsisUnsaved: "你的梗概修改尚未保存。",
  retryAllSynopsisSaves: "重试保存全部梗概",
  retrySaving: "重试保存",
  savingShort: "正在保存…",
  dismissError: "关闭错误提示",
  dismiss: "关闭",
  closeConfirmation: "关闭确认对话框",
};

export default catalog;
