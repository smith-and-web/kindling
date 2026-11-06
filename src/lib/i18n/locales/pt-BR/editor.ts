import type { Catalog } from "../../types";

/** Scene panel, beat list, prose editors and writing statistics. */
const catalog: Catalog = {
  // ScenePanel
  "scenePanel.revisions": "Revisões",
  "scenePanel.openingRevisions": "Abrindo revisões…",
  "scenePanel.revisionsUnsavedError":
    "Salve ou recupere a prosa não salva antes de abrir as Revisões.",
  "scenePanel.viewGroupLabel": "Visualizar",
  "scenePanel.beatsTab": "Beats",
  "scenePanel.beatsTabTooltip": "Cartões de beat",
  "scenePanel.pageTab": "Página",
  "scenePanel.pageTabTooltip": "Prosa em página inteira",
  "scenePanel.sequence": "Sequência",
  "scenePanel.chapter": "Capítulo",
  "scenePanel.breadcrumb": "{kind} · {name}",
  "scenePanel.pageEstimate": "~{pages} páginas",
  "scenePanel.locked": "Bloqueado",
  "scenePanel.sceneTypeLabel": "Tipo de cena",
  "scenePanel.sceneTypeNormal": "Normal",
  "scenePanel.sceneTypeNotes": "Notas",
  "scenePanel.sceneTypeTodo": "A fazer",
  "scenePanel.sceneTypeUnused": "Não utilizado",
  "scenePanel.statusLabel": "Status",
  "scenePanel.sceneStatusDraft": "Rascunho",
  "scenePanel.sceneStatusRevised": "Revisado",
  "scenePanel.sceneStatusFinal": "Final",
  "scenePanel.planningLabel": "Planejamento",
  "scenePanel.planningFixed": "Fixo",
  "scenePanel.planningFlexible": "Flexível",
  "scenePanel.planningUndefined": "Indefinido",
  "scenePanel.planningTooltip":
    "Controla o nível de estrutura desta cena: Indefinido → Flexível → Fixo",
  "scenePanel.aboutPlanning": "Sobre o planejamento",
  "scenePanel.planningHelp":
    "Controla o nível de estrutura desta cena: Indefinido, Flexível ou Fixo.",
  "scenePanel.tagsLabel": "Etiquetas",
  "scenePanel.rollingOutline": "Roteiro contínuo",
  "scenePanel.dismiss": "Dispensar",
  "scenePanel.guideLedeBefore": "O menu ",
  "scenePanel.guidePlanning": "Planejamento",
  "scenePanel.guideLedeAfter":
    " acima controla o nível de estrutura desta cena. Use-o para desenvolver sua história gradualmente:",
  "scenePanel.guideUndefinedDesc": "Um marcador — você sabe que existe, mas ainda não planejou.",
  "scenePanel.guideFlexibleDesc": "Você tem a ideia — uma sinopse e uma direção aproximada.",
  "scenePanel.guideFixedDesc": "Estrutura completa com beats, referências e notas.",
  "scenePanel.undefinedSceneTitle": "Cena indefinida",
  "scenePanel.undefinedSceneBody":
    "Este é um marcador — você sabe que existe, mas ainda não planejou.",
  "scenePanel.undefinedSceneHelp":
    "Adicione uma sinopse abaixo para capturar a ideia geral, depois promova quando estiver pronto para desenvolvê-la.",
  "scenePanel.switchToFlexible": "Mudar para Flexível",
  "scenePanel.goStraightToFixed": "Ir direto para Fixo",
  "scenePanel.flexibleSceneTitle": "Cena flexível",
  "scenePanel.flexibleSceneBody":
    "Você tem uma ideia para esta cena, mas ainda não definiu a estrutura.",
  "scenePanel.flexibleSceneHelp":
    "Use a sinopse para capturar sua intenção. Quando estiver pronto para dividir em beats, mude para Fixo.",
  "scenePanel.defineBeats": "Definir beats",
  "scenePanel.sceneLockedTitle": "Esta cena está bloqueada",
  "scenePanel.chapterLockedHelp":
    "O capítulo pai está bloqueado. Desbloqueie o capítulo para editar esta cena.",
  "scenePanel.sceneLockedHelp": "Desbloqueie esta cena na barra lateral para fazer alterações.",
  "scenePanel.synopsisTitle": "Sinopse",
  "scenePanel.editSynopsis": "Editar sinopse",
  "scenePanel.synopsisPlaceholder": "Escreva uma breve sinopse para esta cena…",
  "scenePanel.saving": "Salvando…",
  "scenePanel.synopsisEditingHelp":
    "Pressione Escape para fechar. As alterações são salvas automaticamente.",
  "scenePanel.addSynopsis": "Adicionar sinopse",
  "scenePanel.sceneLockedShort": "Cena bloqueada",
  "scenePanel.synopsisSaveFailed":
    "Sinopse não salva: {error}. Seu rascunho foi mantido para nova tentativa.",
  "scenePanel.retrySynopsisSave": "Tentar salvar sinopse novamente",
  "scenePanel.retrySaving": "Tentar salvar novamente",
  "scenePanel.unsavedChanges": "Alterações não salvas",
  "scenePanel.referencesTitle": "Referências",
  "scenePanel.loading": "Carregando…",
  "scenePanel.noReferences":
    "Nenhuma referência vinculada ainda. Vincule-as no painel de referências.",
  "scenePanel.discoveryNotesTitle": "Notas de descoberta",
  "scenePanel.hide": "Ocultar",
  "scenePanel.show": "Mostrar",
  "scenePanel.newNoteLabel": "Nova nota de descoberta",
  "scenePanel.newNotePlaceholder": "O que você descobriu?",
  "scenePanel.addingNote": "Adicionando…",
  "scenePanel.addNote": "Adicionar nota",
  "scenePanel.cancel": "Cancelar",
  "scenePanel.editNoteLabel": "Editar nota de descoberta",
  "scenePanel.save": "Salvar",
  "scenePanel.edit": "Editar",
  "scenePanel.promoting": "Promovendo…",
  "scenePanel.promoteToBeat": "Promover para beat",
  "scenePanel.delete": "Excluir",
  "scenePanel.noNotes":
    "Nenhuma nota de descoberta ainda. Capture o que encontrar ao redigir — as notas não fazem parte do manuscrito.",
  "scenePanel.contentTitle": "Conteúdo",
  "scenePanel.referenceLoadError": "Falha ao carregar itens de referência da cena",
  "scenePanel.metadataUpdateError": "Falha ao atualizar metadados da cena",
  "scenePanel.planningUpdateError": "Falha ao atualizar status de planejamento",
  "scenePanel.renameSceneError": "Falha ao renomear cena: {error}",
  "scenePanel.addNoteError": "Falha ao adicionar nota de descoberta: {error}",
  "scenePanel.saveNoteError": "Falha ao salvar nota de descoberta: {error}",
  "scenePanel.deleteNoteError": "Falha ao excluir nota de descoberta: {error}",
  "scenePanel.promoteNoteError": "Falha ao transformar nota em beat: {error}",
  "scenePanel.modeSwitchUnsavedError":
    "Salve, recupere ou descarte a prosa não salva em Buscar e Substituir antes de alternar os modos do editor.",
  "scenePanel.modeSwitchError": "Falha ao alternar modo do editor: {error}",
  "scenePanel.sceneChangedError":
    "O projeto ou a cena selecionada mudou. Reabra Buscar e Substituir.",
  "scenePanel.emptyTitle": "Selecione uma cena para começar a escrever",
  "scenePanel.emptyBody": "Escolha uma cena na barra lateral para ver seu conteúdo.",
  "scenePanel.switchToBeatTitle": "Alternar para Visualização de Beats",
  "scenePanel.switchToBeatMessage":
    "Quaisquer alterações feitas na Visualização de Página serão sincronizadas de volta aos beats correspondentes. Continuar?",
  "scenePanel.switchConfirm": "Alternar",

  // BeatView
  "beatView.beatsTitle": "Beats",
  "beatView.addBeat": "Adicionar beat",
  "beatView.dragToReorder": "Arraste para reordenar",
  "beatView.beatTitleLabel": "Título do beat",
  "beatView.beatWords": "{count} palavras",
  "beatView.beatMenu": "Menu do beat",
  "beatView.sceneLocked": "Cena bloqueada",
  "beatView.prosePlaceholder": "Escreva a prosa deste beat…",
  "beatView.openBeatProse": "Abrir prosa do beat {number}",
  "beatView.emptyTitle": "Nenhum beat ainda",
  "beatView.emptyBody":
    "Os beats vêm de um roteiro importado, ou você pode adicioná-los aqui, um momento de cada vez.",
  "beatView.addFirstBeat": "Adicione seu primeiro beat",
  "beatView.newBeatLabel": "Novo beat",
  "beatView.newBeatPlaceholder": "Descreva o que acontece neste beat…",
  "beatView.newBeatHelp": "Pressione Enter para criar, Escape para cancelar.",
  "beatView.cancel": "Cancelar",
  "beatView.creating": "Criando…",
  "beatView.createBeat": "Criar beat",
  "beatView.deleteTitle": "Excluir beat",
  "beatView.deleteFirstMessage":
    "Tem certeza de que deseja excluir este beat? Este é o primeiro beat, portanto sua prosa será excluída permanentemente.",
  "beatView.deleteMergeMessage":
    "Tem certeza de que deseja excluir este beat? Qualquer prosa será mesclada ao beat anterior.",
  "beatView.menuRename": "Renomear",
  "beatView.menuSplit": "Dividir no cursor",
  "beatView.menuMerge": "Mesclar com o próximo",
  "beatView.menuMoveUp": "Mover para cima",
  "beatView.menuMoveDown": "Mover para baixo",
  "beatView.menuDelete": "Excluir",
  "beatView.createError": "Falha ao criar beat: {error}",
  "beatView.renameError": "Falha ao renomear beat: {error}",
  "beatView.deleteError": "Falha ao excluir beat: {error}",
  "beatView.splitError": "Falha ao dividir beat: {error}",
  "beatView.mergeError": "Falha ao mesclar beats: {error}",
  "beatView.transformUnsavedError":
    "Salve os beats afetados ou recupere seus rascunhos não salvos em Buscar e Substituir primeiro.",
  "beatView.reorderError": "Falha ao reordenar beats: {error}",
  "beatView.moveUpAnnouncement":
    "Beat “{title}” movido para cima, para a posição {position} de {total}.",
  "beatView.moveDownAnnouncement":
    "Beat “{title}” movido para baixo, para a posição {position} de {total}.",

  // ProseToolbar
  "proseToolbar.label": "Formatação de texto",
  "proseToolbar.bold": "Negrito",
  "proseToolbar.italic": "Itálico",
  "proseToolbar.underline": "Sublinhado",
  "proseToolbar.monospace": "Monoespaçado",
  "proseToolbar.alignLeft": "Alinhar à esquerda",
  "proseToolbar.alignCenter": "Centralizar",
  "proseToolbar.alignRight": "Alinhar à direita",
  "proseToolbar.alignJustify": "Justificar",
  "proseToolbar.blockquote": "Citação em bloco",
  "proseToolbar.indent": "Recuar",
  "proseToolbar.undo": "Desfazer",
  "proseToolbar.redo": "Refazer",

  // SluglineInput
  "sluglineInput.sceneHeading": "Cabeçalho de cena",
  "sluglineInput.interiorOrExterior": "Interior ou exterior",
  "sluglineInput.location": "Locação",
  "sluglineInput.locationPlaceholder": "Locação",
  "sluglineInput.timeOfDay": "Hora do dia",

  // WritingStatusBar
  "writingStatusBar.statisticsTitle": "Estatísticas de escrita",
  "writingStatusBar.hideStatistics": "Ocultar estatísticas",
  "writingStatusBar.totalWords": "Total de palavras",
  "writingStatusBar.scenesWithProse": "Cenas com prosa",
  "writingStatusBar.emptyScenes": "Cenas vazias",
  "writingStatusBar.averageWordsPerScene": "Média de palavras por cena",
  "writingStatusBar.savedProseNote":
    "Apenas prosa salva. O conteúdo arquivado é excluído; a média inclui cenas vazias.",
  "writingStatusBar.wordsPerChapter": "Palavras por capítulo",
  "writingStatusBar.chapterColumn": "Capítulo",
  "writingStatusBar.wordsColumn": "Palavras",
  "writingStatusBar.noChapters": "Nenhum capítulo ainda.",
  "writingStatusBar.sceneWords": "Cena: {count} palavras",
  "writingStatusBar.chapterWords": "Capítulo: {count} palavras",
  "writingStatusBar.projectWords": "Projeto: {count} palavras",
  "writingStatusBar.sessionWords": "Sessão: {count} palavras",
  "writingStatusBar.sessionTooltip":
    "Palavras salvas líquidas desde a abertura do aplicativo ou o reinício da sessão.",

  // WritingProgress
  "writingProgress.projectWords": "{count} palavras no projeto",
  "writingProgress.todayTooltip":
    "Palavras adicionadas por edições salvas hoje. Exclusões reduzem esse total.",
  "writingProgress.todayWithGoal": "Hoje: {today} / {goal} palavras",
  "writingProgress.todayWithoutGoal": "Hoje: {today} palavras · meta desativada",
  "writingProgress.dailyGoal": "Meta diária de escrita",
  "writingProgress.sessionWords": "Sessão: {count} palavras",
  "writingProgress.sessionTooltip":
    "Palavras salvas líquidas neste projeto desde a abertura do aplicativo ou o reinício.",
  "writingProgress.resetSession": "Reiniciar sessão de escrita",
  "writingProgress.reset": "Reiniciar",
  "writingProgress.streak.one": "{count} dia de escrita seguido",
  "writingProgress.streak.other": "{count} dias de escrita seguidos",
  "writingProgress.resetError": "Salve sua prosa pendente antes de reiniciar: {error}",

  // NovelEditor
  "novelEditor.placeholder": "Escreva sua prosa…",
  "novelEditor.sceneProse": "Prosa da cena",
  "novelEditor.beatProse": "Prosa do beat",
  "novelEditor.saving": "Salvando…",
  "novelEditor.saveError": "Não foi possível salvar — seu texto ainda está aqui",

  // PageView
  "pageView.title": "Prosa da cena",
  "pageView.words": "{count} palavras",
  "pageView.lockedPlaceholder": "Cena bloqueada",
  "pageView.placeholder": "Escreva a prosa desta cena aqui…",

  // Previously
  "previously.toggle": "Anteriormente",
  "previously.toolsLabel": "Ferramentas da cena",
  "previously.heading": "Anteriormente",
  "previously.loadError": "Não foi possível carregar o contexto da cena anterior.",
  "previously.retry": "Tentar novamente",
};

export default catalog;
