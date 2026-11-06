import type { Catalog } from "../../types";

/** Settings dialog and its Appearance, Author, Keyboard and Project panes. */
const catalog: Catalog = {
  // --- Settings dialog: navigation and chrome ---
  "dialog.title": "Configurações",
  "dialog.closeLabel": "Fechar configurações",
  "dialog.areasAria": "Áreas de configurações",
  "dialog.groupPreferences": "Preferências",
  "dialog.groupProjects": "Projetos",
  "dialog.sectionManuscript": "Manuscrito",
  "dialog.sectionReferenceLibrary": "Biblioteca de Referências",
  "dialog.areaAppearance": "Aparência e Orientação",
  "dialog.areaAuthor": "Autor e Contato",
  "dialog.areaKeyboard": "Atalhos de Teclado",
  "dialog.areaDetails": "Detalhes do Projeto",
  "dialog.areaReferences": "Tipos de Referência",
  "dialog.areaTags": "Etiquetas",
  "dialog.areaFields": "Campos Personalizados",
  "dialog.discardSettings": "Descartar alterações não salvas nas configurações?",
  "dialog.discardProjectSwitch": "Descartar alterações não salvas neste projeto antes de alternar?",
  "dialog.keepEditing": "Continuar editando",
  "dialog.discard": "Descartar alterações",
  "dialog.loadingProjects": "Carregando projetos…",
  "dialog.retryLoadProjects": "Tentar carregar projetos novamente",
  "dialog.noProjects": "Nenhum projeto disponível.",
  "dialog.projectLabel": "Projeto",
  "dialog.projectHelp":
    "Configurações para este projeto. Seu manuscrito aberto permanece no lugar.",
  "dialog.appliesToAll": "Aplica-se a todos os projetos.",
  "dialog.noProjectsHelp":
    "Nenhum projeto ainda. Crie ou importe um projeto para definir suas configurações.",
  "dialog.loadProjectsFailed": "Não foi possível carregar projetos: {error}",

  // --- Appearance ---
  "appearance.lede":
    "Escolha a aparência do kindling e quanto de orientação ele mostra. As alterações se aplicam imediatamente a todos os projetos.",
  "appearance.title": "Aparência",
  "appearance.themeAria": "Tema",
  "appearance.themeDark": "Escuro",
  "appearance.themeLight": "Claro",
  "appearance.themeSystem": "Sistema",
  "appearance.systemHelp":
    "“Sistema” segue a configuração de aparência do seu sistema operacional.",
  "appearance.guidance": "Orientação",
  "appearance.showTips": "Mostrar dicas de orientação",
  "appearance.tipsHelp":
    "Uma dica aparece na primeira vez que você visita a barra lateral, o painel de cena e as referências. Cada dica aparece uma vez.",
  "appearance.showTipsAgain": "Mostrar dicas novamente",
  "appearance.takeTour": "Fazer o tour",
  "appearance.turnOnTips": "Ative as dicas de orientação para mostrá-las novamente.",
  "appearance.tipsReset": "As dicas aparecerão novamente conforme você visita cada área.",

  // --- Author & contact ---
  "author.lede":
    "Seus detalhes de autor e contato se aplicam às páginas de título do manuscrito em todos os projetos.",
  "author.loading": "Carregando detalhes do autor…",
  "author.infoTitle": "Informações do autor",
  "author.nameLabel": "Nome do Autor",
  "author.namePlaceholder": "Seu nome legal",
  "author.nameHelp":
    "Usado nas informações de contato nas páginas de título. Os projetos podem substituir isso por um pseudônimo.",
  "author.contactTitle": "Informações de contato",
  "author.contactHelp":
    "Detalhes opcionais para páginas de título de manuscrito. Use qualquer formato adequado ao seu país.",
  "author.address1": "Linha de Endereço 1",
  "author.address1Placeholder": "Endereço ou caixa postal",
  "author.address2": "Linha de Endereço 2",
  "author.address2Placeholder": "Cidade, Estado/Província, CEP, País",
  "author.phone": "Telefone",
  "author.phonePlaceholder": "+55 (11) 1234-5678",
  "author.email": "E-mail",
  "author.emailPlaceholder": "autor@email.com",
  "author.saved": "Detalhes do autor salvos.",
  "author.save": "Salvar detalhes do autor",
  "author.loadFailedTitle": "Não foi possível carregar os detalhes do autor",
  "author.loadFailedHelp":
    "Se o arquivo de configurações estiver corrompido, redefina os detalhes do autor para restaurar o salvamento e as exportações. Uma cópia de recuperação do arquivo original será mantida.",
  "author.retry": "Tentar carregar detalhes do autor novamente",
  "author.reset": "Redefinir detalhes do autor",

  // --- Keyboard shortcuts ---
  "keyboard.lede":
    "Escolha um atalho, em seguida pressione uma nova combinação de teclas. As alterações se aplicam imediatamente a todos os projetos. Escape cancela a gravação; Tab passa para o próximo controle.",
  "keyboard.systemHelp":
    "Os controles padrão de edição de texto e do sistema (como copiar, colar, desfazer, Tab e Escape) mantêm suas teclas usuais.",
  "keyboard.retryLoad": "Tentar carregar atalhos novamente",
  "keyboard.filterLabel": "Filtrar comandos",
  "keyboard.resetDefaults": "Redefinir todos para o padrão",
  "keyboard.loading": "Carregando atalhos…",
  "keyboard.shortcutFor": "Atalho para {label}",
  "keyboard.pressKeys": "Pressione as teclas…",
  "keyboard.unassigned": "Não atribuído",
  "keyboard.clearFor": "Limpar atalho para {label}",
  "keyboard.clear": "Limpar",
  "keyboard.noMatches": "Nenhum comando correspondente.",
  "keyboard.saved": "Atalhos de teclado salvos.",
  "keyboard.saveFailed": "Não foi possível salvar os atalhos de teclado: {error}",
  "keyboard.invalidKey":
    "Use Command no macOS ou Ctrl no Windows/Linux, com uma letra, número, tecla de pontuação ou F1–F24.",

  // --- Project details ---
  "project.ledePrefix": "As alterações aqui se aplicam apenas a ",
  "project.ledeSuffix": ".",
  "project.tagChanges": "As alterações de etiquetas são salvas à medida que você as faz.",
  "project.fieldChanges":
    "As alterações em campos personalizados são salvas à medida que você as faz. Os campos são listados para tipos de referência salvos e ativados.",
  "project.optional": "(opcional)",
  "project.penName": "Pseudônimo",
  "project.penNamePlaceholder": "Deixe em branco para usar seu nome de autor",
  "project.penNameHelp":
    "Se fornecido, será usado como assinatura nas páginas de título em vez do seu nome de autor.",
  "project.genre": "Gênero",
  "project.genrePlaceholder": "ex.: Ficção literária, Ficção científica, Mistério",
  "project.genreHelp": "O gênero será exibido nas páginas de título do manuscrito.",
  "project.description": "Descrição do Projeto",
  "project.descriptionPlaceholder": "Resumo curto ou notas sobre este projeto",
  "project.dailyGoal": "Meta diária de escrita",
  "project.loadDailyGoalFailed": "Não foi possível carregar a meta diária: {error}",
  "project.dailyGoalHelp":
    "Palavras adicionadas por dia neste projeto. Defina como 0 para desativar a meta. As alterações se aplicam hoje; os dias anteriores da sequência mantêm suas metas originais.",
  "project.wordTarget": "Meta de Palavras",
  "project.wordTargetPlaceholder": "ex.: 80000",
  "project.referencesHelp":
    "Escolha quais tipos de referência aparecem no painel de Referências deste projeto. Desativar um tipo mantém suas entradas existentes.",
  "project.enableRefTypes": "Ative os tipos de referência para configurar campos personalizados.",
  "project.saved": "Alterações do projeto salvas.",
  "project.save": "Salvar alterações do projeto",
  "project.wordTargetInvalid": "A meta de palavras deve ser um número inteiro não negativo",
  "project.dailyGoalInvalid": "A meta diária deve ser um número inteiro entre 0 e 1.000.000",
  "project.disabledDraft":
    "Salve ou cancele o rascunho do campo personalizado {label} antes de desativar esse tipo de referência.",
};

export default catalog;
