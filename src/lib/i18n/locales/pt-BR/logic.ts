import type { Catalog } from "../../types";

/** Copy produced outside components: stores, utils, import formats and command labels. */
const catalog: Catalog = {
  // --- UI store ---
  "ui.importing": "Importando",
  "ui.startingImport": "Iniciando importação…",
  "ui.importComplete": "Importação concluída!",

  // --- Shortcut store and binding checks ---
  "shortcuts.loadFailed": "Não foi possível carregar os atalhos de teclado: {error}",
  "shortcuts.updateFailed": "Não foi possível atualizar os atalhos de teclado nativos: {error}",
  "shortcuts.reserved":
    "Este atalho é reservado para edição de texto padrão ou controles de janela.",
  "shortcuts.conflict": "Já usado por {label}. Limpe ou altere esse atalho primeiro.",

  // --- Import helpers ---
  "import.selectScrivenerFolder": "Selecionar pasta do projeto Scrivener (.scriv)",
  "import.scrivenerFolderOnly": "Selecione uma pasta cujo nome termine com .scriv",
  "import.unknownType": "Tipo de importação desconhecido: {type}",
  "import.failed": "Importação falhou: {error}",

  // --- Import formats ---
  "importFormat.plottr": "Arquivo Plottr",
  "importFormat.markdown": "Arquivo Markdown",
  "importFormat.ywriter": "Arquivo yWriter",
  "importFormat.longform": "Índice Longform",
  "importFormat.longformVault": "Vault Longform",
  "importFormat.novelwriter": "Projeto novelWriter",
  "importFormat.scrivener": "Projeto Scrivener",

  // --- Reference types ---
  "referenceType.characters": "Personagens",
  "referenceType.locations": "Locações",
  "referenceType.items": "Itens",
  "referenceType.objectives": "Objetivos",
  "referenceType.organizations": "Organizações",
  "referenceType.timelines": "Linhas do tempo",
  "referenceType.custom": "Notas",

  // --- Editorial review validation ---
  "editorial.unknownSource": "A revisão contém uma fonte de manuscrito desconhecida.",
  "editorial.paragraphFormatting": "A revisão contém formatação de parágrafo não suportada.",
  "editorial.unsupportedLink": "A revisão contém um link não suportado.",
  "editorial.outsidePassage":
    "Uma anotação de feedback refere-se a uma passagem fora deste manuscrito.",
  "editorial.mismatchedPassage":
    "Uma anotação de feedback não corresponde à sua passagem original.",
  "editorial.invalidAnchor": "Um comentário tem uma âncora de passagem inválida.",
  "editorial.noSources": "A revisão não tem fontes de manuscrito.",
  "editorial.reorderSources":
    "Esta edição reordenaria as fontes do manuscrito. Copie a passagem como texto.",
  "editorial.lostSource":
    "Um parágrafo do manuscrito perdeu sua fonte. Reabra a revisão antes de continuar.",
  "editorial.noCurrentPassages":
    "Não há passagens atuais do manuscrito nesta revisão. Restaure uma cena antes de aplicar o feedback.",
  "editorial.passageChanged":
    "Esta passagem foi alterada. Compare a prosa original, atual e sugerida e selecione onde a alteração pertence.",
  "editorial.overlap": "Essas sugestões se sobrepõem. Revise-as e aceite-as individualmente.",
  "editorial.selectPassage": "Selecione uma passagem dentro do manuscrito.",

  // --- Prose save queue ---
  "proseSaves.draftsChanged":
    "Os rascunhos não salvos foram alterados. Tente carregar novamente antes de descartá-los.",
  "proseSaves.flushFailed":
    "Não foi possível salvar a prosa mais recente. {errors} Tente salvar novamente ou revise os rascunhos não salvos em Buscar e Substituir.",
  "proseFlush.message":
    "Salve ou recupere a prosa não salva antes de {action}. Revise os rascunhos não salvos em Buscar e Substituir.",
  "proseFlush.action.exporting": "exportar",
  "proseFlush.action.syncing": "sincronizar",

  // --- Editorial recovery and re-anchoring ---
  "editorialSaves.recoveredMismatch":
    "A revisão recuperada tem um manuscrito diferente. Mantenha esta janela aberta e exporte uma cópia de recuperação.",
  "localEditorial.reanchor":
    "Selecione uma passagem dentro da prosa original desta cena para reancorar sua anotação salva.",

  // --- Review suggestions ---
  "revisions.suggestionClosed": "A sugestão não está mais aberta",
  "revisions.proseChanged":
    "A prosa foi alterada. Reancoreie sugestões desatualizadas antes de aceitá-las.",
  "revisions.overlap": "As sugestões se sobrepõem. Revise-as e aceite-as individualmente.",

  // --- Utility errors ---
  "referenceCopy.failed": "Não foi possível copiar as referências. Tente novamente.",
  "synopsisSaves.changed":
    "Os rascunhos de sinopse foram alterados. Revise as últimas alterações antes de descartá-los.",
  "exportPrototype.nameRequired": "Dê um nome a este perfil antes de salvar.",
  "exportPrototype.startNumberInteger": "O número inicial do capítulo deve ser um número inteiro.",
  "exportPrototype.unsupportedFormat":
    "Os perfis de protótipo salvos têm um formato não suportado.",
  "exportPrototype.invalidProfile": "Um perfil de protótipo salvo é inválido.",
  "exportPrototype.unsupportedOption": "Uma opção de protótipo salva não é suportada.",
  "exportPrototype.duplicateIds": "Os perfis de protótipo salvos têm IDs duplicados.",
  "exportPrototype.fontSize": "Tamanho da fonte",
  "exportPrototype.lineSpacing": "Espaçamento de linha",
  "exportPrototype.paragraphSpacing": "Espaçamento de parágrafo",
  "exportPrototype.indent": "Recuo da primeira linha",
  "exportPrototype.margin": "Margens",
  "exportPrototype.startNumber": "Número inicial do capítulo",
  "exportPrototype.rangeError": "{label} deve estar entre {min} e {max}.",

  // --- Command categories ---
  "commandCategory.File": "Arquivo",
  "commandCategory.View": "Visualizar",
  "commandCategory.Edit": "Editar",
  "commandCategory.Help": "Ajuda",
  "commandCategory.Project": "Projeto",
  "commandCategory.Editor": "Editor",

  // --- Command labels (mirror shortcutDefinitions.json) ---
  "command.editorial_open.label": "Abrir arquivo de revisão ou feedback",
  "command.editorial_project.label": "Revisão editorial e pacotes",
  "command.find.label": "Buscar na cena",
  "command.find_replace.label": "Buscar e substituir",
  "command.find_project.label": "Buscar e substituir no projeto",
  "command.export.label": "Exportar projeto",
  "command.close_project.label": "Fechar projeto",
  "command.quit.label": "Sair do kindling",
  "command.import_plottr.label": "Importar Plottr (.pltr)",
  "command.import_markdown.label": "Importar Markdown (.md)",
  "command.import_longform.label": "Importar Longform",
  "command.import_ywriter.label": "Importar yWriter 7 (.yw7)",
  "command.import_scrivener.label": "Importar Scrivener 3 (.scriv)",
  "command.import_novelwriter.label": "Importar novelWriter",
  "command.sync.label": "Sincronizar da fonte",
  "command.toggle_sidebar.label": "Alternar barra lateral",
  "command.toggle_references.label": "Alternar painel de referências",
  "command.toggle_discovery_notes.label": "Alternar notas de descoberta",
  "command.toggle_editor_mode.label": "Alternar visualização de beat/página",
  "command.detect_references.label": "Detectar referências na cena",
  "command.detect_all_references.label": "Detectar referências em todas as cenas",
  "command.about.label": "Sobre o kindling",
  "command.quick_start.label": "Guia de início rápido",
  "command.settings.label": "Configurações",
  "command.new_project.label": "Novo projeto",
  "command.command_palette.label": "Paleta de comandos",
  "command.send_feedback.label": "Enviar feedback",
  "command.editorial_comment.label": "Adicionar comentário editorial",
  "command.bold.label": "Negrito",
  "command.italic.label": "Itálico",
  "command.underline.label": "Sublinhado",
  "command.strike.label": "Tachado",
  "command.code.label": "Monoespaçado",
  "command.blockquote.label": "Citação em bloco",
  "command.align_left.label": "Alinhar à esquerda",
  "command.align_center.label": "Centralizar",
  "command.align_right.label": "Alinhar à direita",
  "command.align_justify.label": "Justificar",
};

export default catalog;
