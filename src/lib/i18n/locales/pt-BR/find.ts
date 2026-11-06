import type { Catalog } from "../../types";

/** Find and Replace dialog. */
const catalog: Catalog = {
  "findReplaceDialog.title": "Buscar e Substituir",
  "findReplaceDialog.find": "Buscar",
  "findReplaceDialog.previousMatch": "Correspondência anterior (Shift+Enter)",
  "findReplaceDialog.previous": "Anterior",
  "findReplaceDialog.nextMatch": "Próxima correspondência (Enter)",
  "findReplaceDialog.next": "Próximo",
  "findReplaceDialog.loadingProse": "Carregando prosa…",
  "findReplaceDialog.enterQuery": "Insira o texto a buscar.",
  "findReplaceDialog.searchIn": "Buscar em",
  "findReplaceDialog.currentScene": "Cena atual",
  "findReplaceDialog.entireProject": "Projeto inteiro",
  "findReplaceDialog.matchCase": "Diferenciar maiúsculas",
  "findReplaceDialog.wholeWords": "Palavras inteiras",
  "findReplaceDialog.replace": "Substituir",
  "findReplaceDialog.replaceWith": "Substituir por",
  "findReplaceDialog.scopeHelp":
    "Busca a prosa visível em cenas Fixas. Cenas Flexíveis, Indefinidas e arquivadas são excluídas; cenas bloqueadas são pesquisáveis, mas não podem ser substituídas.",
  "findReplaceDialog.position": "{current} de {total}",
  "findReplaceDialog.locked": " · Bloqueado",
  "findReplaceDialog.unsavedDraft": " · Rascunho não salvo",
  "findReplaceDialog.openScene": "Abrir cena",
  "findReplaceDialog.noMoreMatches":
    "Sem mais correspondências à frente. Use Próximo ou Anterior para continuar.",
  "findReplaceDialog.noMatches": "Nenhuma correspondência encontrada.",
  "findReplaceDialog.confirmReplace":
    "Substituir {count} correspondências em {scope}? {skipped} correspondências bloqueadas ou não salvas serão ignoradas.",
  "findReplaceDialog.scopeEntireProject": "o projeto inteiro",
  "findReplaceDialog.scopeCurrentScene": "a cena atual",
  "findReplaceDialog.confirmReplaceAll": "Confirmar substituição em massa",
  "findReplaceDialog.retryLoading": "Tentar carregar novamente",
  "findReplaceDialog.retrySavingDrafts": "Tentar salvar rascunhos novamente",
  "findReplaceDialog.draftsHelp":
    "Os rascunhos não salvos são retidos para esta sessão, incluindo após fechar o projeto. Documentos bloqueados ou ausentes e erros de salvamento não reconhecidos não são tentados novamente automaticamente e não bloqueiam outras cenas. Você pode tentar novamente após resolver o erro de salvamento, ou copiar esses rascunhos antes de descartá-los. Correspondências em documentos com rascunhos não salvos não podem ser substituídas.",
  "findReplaceDialog.unsavedScene": "Cena não salva {index}",
  "findReplaceDialog.unsavedBeat": "Beat não salvo {index}",
  "findReplaceDialog.draftText": "Texto do rascunho (selecione para copiar)",
  "findReplaceDialog.discardQuestion":
    "Descartar estes rascunhos não salvos? Suas alterações serão perdidas. Copie o texto que deseja manter primeiro.",
  "findReplaceDialog.confirmDiscardDrafts": "Confirmar descarte dos rascunhos",
  "findReplaceDialog.keepDrafts": "Manter rascunhos",
  "findReplaceDialog.discardDrafts": "Descartar rascunhos não salvos…",
  "findReplaceDialog.undoReplacement": "Desfazer substituição",
  "findReplaceDialog.replaceAll": "Substituir tudo",
  "findReplaceDialog.replaceMatch": "Substituir correspondência",
  "findReplaceDialog.replacementUndone": "Substituição desfeita.",
  "findReplaceDialog.replacementSaved": "Substituição salva.",
  "findReplaceDialog.noChangesNeeded": "Nenhuma alteração necessária.",
};

export default catalog;
