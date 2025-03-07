import { Translation } from "./translations";

export const ptPt: Translation = {
	meta: {
		nativeName: "Português (Portugal)",
	},
	tab: {
		games: "Jogos",
		mods: "Mods",
		tools: "Ferramentas",
		thanks: "Agradecimentos",
	},

	addGame: {
		button: "Adicionar jogo",
		title: "Adicionar jogo",
		dropField:
			"Arrasta e larga um executável de jogo aqui ou clica para selecionar um ficheiro.",
		note: "Nota: podes largar ficheiros executáveis de jogos em qualquer lugar na janela do Rai Pal para os adicionar à lista de jogos instalados sem abrir este diálogo.",
	},

	refresh: {
		button: "Atualizar",
		loading: "A carregar {items}...",
	},

	filterMenu: {
		button: "Filtrar",
		resetButton: "Repor",
		searchPlaceholder: "Procurar...",
	},

	filterProperty: {
		provider: "Fornecedor",
		tags: "Etiqueta",
		architecture: "Arquitetura",
		unityScriptingBackend: "Backend do Unity",
		engine: "Motor",
		status: "Estado",
	},

	filterValue: {
		unknown: "Desconhecido",
		arch64: "64-bit",
		arch32: "32-bit",
		tagDemo: "Demo",
		tagVr: "VR Nativo",
		tagUntagged: "Sem etiqueta",
		statusInstalled: "Instalado",
		statusNotInstalled: "Não instalado",
		providerManual: "Manual",
	},

	filterValueNote: {
		providerXboxOnlyInstalled:
			"Os jogos Xbox para PC só aparecem no Rai Pal depois de serem instalados.",
		engineGodotNotFullySupported:
			"Os jogos Godot ainda não são totalmente suportados.",
		engineGameMakerNotFullySupported:
			"Os jogos GameMaker ainda não são totalmente suportados.",
	},

	providerCommand: {
		installGame: "Instalar",
		showGameInLibrary: "Mostrar na Biblioteca",
		showGameInStore: "Abrir Página da Loja",
		startGame: "Iniciar Jogo",
		openGamePageInBrowser: "Abrir no Navegador",
	},

	gameModal: {
		startGameButton: "Iniciar Jogo",
		startGameExecutable: "Iniciar Executável do Jogo",
		startGameViaProvider: "Iniciar Jogo via {provider}",
		foldersDropdown: "Pastas",
		openGameFilesFolder: "Abrir Pasta de Ficheiros do Jogo",
		openInstalledModsFolder: "Abrir Pasta de Mods Instalados",
		removeFromRaiPal: "Remover do Rai Pal",
		removeGameConfirmation:
			"Tens a certeza de que queres remover este jogo do Rai Pal?",
		refreshGame: "Atualizar",
		failedToReadGameInfo:
			"Falha ao ler informações importantes sobre este jogo. O executável pode estar protegido. Alguns mods podem não conseguir ser instalados.",
		failedToDetermineEngine:
			"Falha ao determinar o motor deste jogo. Alguns mods podem não funcionar.",
		gameModsLabel: "Mods",
		gameNotInstalledWarning:
			"Este jogo não está instalado, por isso não tenho 100% de certeza sobre a compatibilidade dos mods. Os que vês abaixo podem funcionar. Se instalares o jogo, poderei mostrar informações mais precisas.",
		uninstallAllModsButton: "Desinstalar todos os mods",
		uninstallAllModsConfirmation:
			"Tens a certeza? Isto apagará todos os ficheiros na pasta de mods deste jogo. No entanto, não removerá ficheiros do próprio jogo.",
	},

	gamesTableColumn: {
		game: "Jogo",
		engine: "Motor",
		date: "Data",
	},

	modsPage: {
		openModsFolderButton: "Abrir Pasta de Mods",
		tableColumnMod: "Mod",
		tableColumnVersion: "Versão",
		tableColumnModLoader: "Loader",
		tableColumnGameEngine: "Motor",
		tableColumnUnityBackend: "Backend",
		modByAuthor: "por {authorName}",
		modDeprecated: "Obsoleto",
		modDeprecatedTooltip:
			"Este mod está obsoleto. Deves desinstalá-lo e instalar uma alternativa mais recente.",
		modOutdated: "Mod desatualizado",
	},

	modModal: {
		runMod: "Executar",
		openModFolder: "Abrir pasta do mod",
		updateMod: "Atualizar mod",
		downloadMod: "Descarregar mod",
		deleteMod: "Eliminar mod",
		deleteModConfirmation:
			"Tens a certeza? Todos os ficheiros dentro da pasta do mod serão apagados.",
		byAuthor: "por {authorName}",
	},

	toolsPage: {
		openLogsFolderButton: "Abrir Pasta de Registos",
		resetRaiPalSettingsButton: "Repor definições do Rai Pal",
		resetRaiPalSettingsTooltip:
			"Irá repor filtros, diálogos de confirmação e outras definições.",
		clearCacheButton: "Limpar cache do Rai Pal",
		clearCacheTooltip: "Limpa a cache da lista de jogos usada pelo Rai Pal.",
	},

	appSettings: {
		showGameThumbnails: "Mostrar imagens na lista de jogos",
		language: "Idioma",
		autoDetectedLanguage: "Auto-detetado - {languageName}",
	},

	steamCache: {
		resetSteamCacheButton: "Repor cache da Steam",
		resetSteamCacheModalTitle: "Repor cache da Steam",
		resetSteamCacheDescription:
			"Usa isto se o Rai Pal estiver a mostrar jogos que não possuis na Steam. Isto irá repor a cache da Steam e terás de a reiniciar. Irás receber um erro se o ficheiro já tiver sido apagado.",
		resetSteamCacheSuccess:
			"O ficheiro de cache foi apagado. Reinicia a Steam, espera alguns segundos e depois clica no botão de atualização no Rai Pal.",
	},

	debugData: {
		debugDataTitle: "Dados de Depuração",
		debugDataCopy: "Copiar dados de depuração",
	},

	thanksPage: {
		intro:
			"Olá. Eu fiz o Rai Pal. Também fiz outros mods VR no passado e estou a trabalhar num mod VR universal para jogos Unity. Se gostas do que faço e queres ver mais, considera doar! Também podes apoiar-me comprando um dos meus mods gratuitos no itch.io.",
		starRaiPalOnGitHub: "Dá uma estrela ao Rai Pal no GitHub",
		otherModdersTitle: "Outros modders",
		otherModdersDescription:
			"O Rai Pal ajuda-te a gerir mods, e não seria possível sem as ferramentas de outros programadores. Alguns não têm links de doação, mas estou extremamente grato pelo seu trabalho.",
		modderOnWebsite: "{modderName} no {website}",
		patreonLeaderboard: "Ranking do Patreon",
		rankedByPatreonDonationAmount: "Ordenado pelo montante total doado.",
		patreonProfilePrivateNotice:
			"Se não te vês aqui, é porque o teu perfil no Patreon é privado.",
	},

	commandButton: {
		cancel: "Cancelar",
		dontAskAgain: "Não perguntar novamente",
	},
};
