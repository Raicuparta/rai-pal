export const ptPt = {
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
			"Arrasta e larga um executável de jogo aqui, ou clica para selecionar um ficheiro.",
		note: "Nota: podes largar executáveis de jogo em qualquer parte da janela do Rai Pal sem abrir este diálogo.",
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
		provider: "Provedor",
		tags: "Etiqueta",
		architecture: "Arquitetura",
		unityScriptingBackend: "Backend Unity",
		engine: "Motor",
		status: "Estado",
	},

	filterValue: {
		unknown: "Desconhecido",
		arch64: "64-bit",
		arch32: "32-bit",
		engineGodot: "Godot",
		engineGameMaker: "GameMaker",
		engineUnity: "Unity",
		engineUnreal: "Unreal",
		unityBackendIl2Cpp: "IL2CPP",
		unityBackendMono: "Mono",
		tagDemo: "Demo",
		tagVr: "VR Nativo",
		tagUntagged: "Sem etiqueta",
		statusInstalled: "Instalado",
		statusNotInstalled: "Não instalado",
		providerSteam: "Steam",
		providerGog: "GOG",
		providerEpic: "Epic",
		providerItch: "itch.io",
		providerOrigin: "Origin",
		providerManual: "Manual",
		providerXbox: "Xbox",
		providerEa: "EA",
		providerUbisoft: "Ubisoft",
	},

	filterValueNote: {
		providerXboxOnlyInstalled:
			"Os jogos do Xbox PC só aparecem no Rai Pal depois de instalados.",
		engineGodotNotFullySupported:
			"Os jogos Godot não são totalmente suportados ainda.",
		engineGameMakerNotFullySupported:
			"Os jogos GameMaker não são totalmente suportados ainda.",
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
		startGameExecutable: "Executável do Jogo",
		startGameViaProvider: "Iniciar Jogo via {provider}",
		foldersDropdown: "Pastas",
		openGameFilesFolder: "Abrir Pasta de Ficheiros do Jogo",
		openInstalledModsFolder: "Abrir Pasta de Mods Instalados",
		removeGameConfirmation:
			"Tens a certeza de que queres remover este jogo do Rai Pal?",
		removeFromRaiPal: "Remover do Rai Pal",
		refreshGame: "Atualizar",
		failedToReadGameInfo:
			"Falhou a leitura de informações importantes sobre este jogo. Pode ser devido a ficheiros protegidos. Alguns mods podem falhar na instalação.",
		failedToDetermineEngine:
			"Falhou a deteção do motor deste jogo. Alguns mods podem falhar na instalação.",
		gameModsLabel: "Mods",
		gameNotInstalledWarning:
			"Este jogo não está instalado; não consigo ter 100% de certeza de quais mods são compatíveis. Se o instalares, vou conseguir mostrar-te informações mais exatas.",
		uninstallAllModsConfirmation:
			"Tens a certeza? Isto vai eliminar todos os ficheiros na pasta de mods deste jogo. Não elimina ficheiros do próprio jogo.",
		uninstallAllModsButton: "Desinstalar todos os mods",
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
		tableColumnModLoader: "Carregador",
		tableColumnGameEngine: "Motor",
		tableColumnUnityBackend: "Backend",
		modByAuthor: "por {authorName}",
		modDeprecated: "Descontinuado",
		modDeprecatedTooltip:
			"Este mod está descontinuado. Deves desinstalá-lo e instalar uma alternativa mais recente.",
		modOutdated: "Mod desatualizado",
	},

	modModal: {
		runMod: "Executar",
		openModFolder: "Abrir pasta do mod",
		updateMod: "Atualizar mod",
		downloadMod: "Transferir mod",
		deleteMod: "Eliminar mod",
		deleteModConfirmation:
			"Tens a certeza? Todos os ficheiros na pasta deste mod vão perder-se.",
		byAuthor: "por {authorName}",
	},

	toolsPage: {
		openLogsFolderButton: "Abrir Pasta de Registos",
		resetRaiPalSettingsButton: "Repor definições do Rai Pal",
		resetRaiPalSettingsTooltip:
			"Vai repor filtros, caixas de confirmação e possivelmente outras definições.",
		clearCacheButton: "Limpar cache do Rai Pal",
		clearCacheTooltip: "Limpa a cache da lista de jogos usada pelo Rai Pal.",
	},

	steamCache: {
		resetSteamCacheButton: "Repor cache da Steam",
		resetSteamCacheModalTitle: "Repor cache da Steam",
		resetSteamCacheDescription:
			"Usa isto se o Rai Pal está a mostrar jogos que não possuis na Steam. Isto vai repor a cache da Steam e depois terás de reiniciar a Steam. Vais receber um erro se o ficheiro já tiver sido eliminado.",
		resetSteamCacheSuccess:
			"O ficheiro de cache foi eliminado. Reinicia a Steam, espera uns segundos e carrega no botão de atualizar no Rai Pal.",
	},

	debugData: {
		debugDataTitle: "Dados de depuração",
		debugDataCopy: "Copiar dados de depuração",
	},

	thanksPage: {
		intro:
			"Olá. Eu criei o Rai Pal. Também criei outros mods de VR e neste momento estou a trabalhar num mod universal de VR para jogos Unity. Se gostas do que faço e queres ver mais, considera fazer uma doação! Podes apoiar-me também comprando um dos meus mods gratuitos no itch.io.",
		starRaiPalOnGitHub: "Dá uma estrela ao Rai Pal no GitHub",
		otherModdersTitle: "Outros criadores de mods",
		otherModdersDescription:
			"O Rai Pal ajuda-te a gerir mods de jogos e não seria possível sem as ferramentas que outros criadores desenvolveram. Alguns não têm links de doação, mas estou muito grato pelo seu trabalho.",
		modderOnWebsite: "{modderName} em {website}",
		patreonLeaderboard: "Ranking de Patreon",
		rankedByPatreonDonationAmount:
			"Classificado pelo valor total de doações no Patreon.",
		patreonProfilePrivateNotice:
			"Se não te vês aqui, é porque o teu perfil no Patreon é privado.",
	},

	commandButton: {
		cancel: "Cancelar",
		dontAskAgain: "Não voltar a perguntar",
	},
} as const;
