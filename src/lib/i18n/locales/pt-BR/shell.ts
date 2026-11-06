import type { Catalog } from "../../types";

/** App shell: quit flow, save failures, toasts and dialog chrome. */
const catalog: Catalog = {
  quitReviewTitle: "Sair sem salvar as alterações da revisão?",
  quitWritingTitle: "Sair sem salvar as alterações de escrita?",
  quitSynopsisTitle: "Sair sem salvar as alterações da sinopse?",
  quitReviewMessage:
    "Sua revisão não pôde ser salva. Continue editando para tentar novamente ou exporte uma cópia de recuperação. Sair e descartar remove todas as alterações não salvas de revisão, prosa e sinopse.",
  quitWritingMessage:
    "Algumas alterações de prosa ou sinopse não puderam ser salvas. Sair e descartar estas alterações de escrita não salvas, ou continuar editando para tentar salvar novamente.",
  quitSynopsisMessage:
    "Algumas alterações de sinopse não puderam ser salvas. Sair e descartar estas alterações de sinopse não salvas, ou continuar editando para tentar salvar novamente.",
  quitAndDiscard: "Sair e descartar",
  keepEditing: "Continuar editando",
  couldNotQuit: "Não foi possível sair: {error}",
  sceneUnavailable: "Esta cena não está mais disponível.",
  proseUnsaved:
    "As alterações de prosa não puderam ser salvas. Revise os rascunhos não salvos em Buscar e Substituir.",
  synopsisUnsaved: "As alterações da sua sinopse não foram salvas.",
  retryAllSynopsisSaves: "Tentar salvar todas as sinopses novamente",
  retrySaving: "Tentar salvar novamente",
  savingShort: "Salvando…",
  dismissError: "Dispensar erro",
  dismiss: "Dispensar",
  closeConfirmation: "Fechar confirmação",
};

export default catalog;
