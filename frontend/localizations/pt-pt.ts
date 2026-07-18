import { Localization } from "./localizations";

export const ptPt: Localization = {
	meta: {
		nativeName: "Português (Portugal)",
	},
	tab: {
		games: "Jogos",
		mods: "Mods",
		thanks: "Agradecimentos",
	},

	gamesPage: {
		emptyGamesList:
			"O Rai Pal não encontrou nenhum jogo. Lembra-te que o Rai Pal precisa de encontrar aplicações instaladas de outros fornecedores de jogos, como Steam, Epic, etc.",
		emptyFilteredGamesList:
			"Nada! Todos os teus jogos estão escondidos devido aos filtros que selecionaste. Limpa os teus filtros para veres os teus belos jogos novamente.",
		emptyGamesLoading: "A procurar os teus jogos...",
	},

	addGame: {
		button: "Adicionar jogo",
		title: "Adicionar jogo",
		dropField:
			"Arrasta e larga um executável de jogo aqui ou clica para selecionar um ficheiro.",
		directoryButton: "Scan a folder recursively for games. Can be slow!",
		note: "Nota: podes largar ficheiros executáveis de jogos em qualquer lugar na janela do Rai Pal para os adicionar à lista de jogos instalados sem abrir este diálogo.",
	},

	refresh: {
		button: "Atualizar",
		buttonUpdateRemoteDatabases: "Atualizar bases de dados remotas",
		loading: "A carregar {items}...",
	},

	filterMenu: {
		button: "Filtrar",
		resetButton: "Repor",
		searchPlaceholder: "Procurar...",
	},

	filterProperty: {
		provider: "Plataforma",
		tags: "Tag",
		architecture: "Arquitetura",
		unityBackend: "Backend do Unity",
		engine: "Motor",
		status: "Estado",
		mod: "Mod",
	},

	filterValue: {
		unknown: "Desconhecido",
		arch64: "64-bit",
		arch32: "32-bit",
		tagDemo: "Demo",
		tagVr: "VR nativo",
		tagUntagged: "Sem tag",
		statusInstalled: "Instalado",
		statusNotInstalled: "Não instalado",
		providerManual: "Manual",
	},

	filterValueNote: {
		providerXboxOnlyInstalled:
			"Só mostra jogos Xbox PC se estiverem instalados.",
		engineGodotNotFullySupported:
			"Os jogos Godot ainda não são totalmente suportados.",
		engineGameMakerNotFullySupported:
			"Os jogos GameMaker ainda não são totalmente suportados.",
	},

	providerCommand: {
		installGame: "Instalar",
		showGameInLibrary: "Mostrar na biblioteca",
		showGameInStore: "Abrir página da loja",
		startGameViaProvider: "Iniciar jogo",
		startGameViaExe: "Abrir executável do jogo",
		openGamePageInBrowser: "Abrir no navegador",
	},

	gameModal: {
		startGameButton: "Iniciar jogo",
		startGameExecutable: "Iniciar executável do jogo",
		startGameViaProvider: "Iniciar jogo via {provider}",
		foldersDropdown: "Pastas",
		openGameFilesFolder: "Ficheiros do jogo",
		openInstalledModsFolder: "Mods instalados",
		openGameDataFolder: "Dados da aplicação",
		openGameWinePrefixFolder: "Prefixo Wine",
		openGameWineBinaryFolder: "Binário Wine",
		removeFromRaiPal: "Remover do Rai Pal",
		removeGameConfirmation: "Queres mesmo remover este jogo do Rai Pal?",
		refreshGame: "Atualizar",
		failedToReadGameInfo:
			"Erro ao ler informações importantes sobre este jogo. O executável pode estar protegido. Alguns mods podem não instalar-se.",
		failedToDetermineEngine:
			"Não foi possível determinar o motor deste jogo. Alguns mods podem não funcionar.",
		gameModsLabel: "Mods",
		gameNotInstalledWarning:
			"Este jogo não está instalado, por isso não tenho 100% de certeza sobre a compatibilidade dos mods. Os que vês abaixo podem funcionar. Se instalares o jogo, posso mostrar informações mais exatas.",
		uninstallAllModsButton: "Desinstalar todos os mods",
		uninstallAllModsConfirmation:
			"Tens a certeza? Isto vai apagar todos os ficheiros da pasta de mods deste jogo. Mas não mexe nos ficheiros do jogo em si.",

		incompatibleGameModsLabel: "Mods Incompatíveis",

		incompatibleGameModsDescription:
			"Os mods listados aqui não podem ser instalados porque não são compatíveis com a versão do motor deste jogo.",
		otherThings: "Outras coisas",
		otherThingsDescription:
			"Estes são principalmente dependências e outras coisas que normalmente não precisas de mexer.",
	},

	gameModRow: {
		editModConfig: "Editar configuração do mod",
		openModConfigFolderTooltip:
			"Abrir a pasta que contém os ficheiros de configuração deste mod",
		openModFolder: "Abrir pasta do mod",
		updateMod: "Atualizar",
		installMod: "Instalar",
		installModAnticheatWarning:
			"Atenção: tem cuidado ao instalar mods em jogos multiplayer! O anticheat pode detetar alguns mods e podes ser banido, mesmo que os mods pareçam inofensivos.",
		reinstallMod: "Reinstalar",
		uninstallMod: "Desinstalar",
		runMod: "Executar",
		downloadRemoteConfig: "Descarregar configuração recomendada",
		remoteConfigAvailable:
			"Configuração recomendada disponível. Será descarregada se ainda não tiveres uma configuração. Também podes forçar o download a partir do menu de três pontos.",
		modOutdated: "Mod desatualizado",
		cantUninstallModWithDependants:
			"Não é possível desinstalar um mod que tenha dependentes. Desinstala primeiro os mods que dependem deste.",
	},

	gamesTableColumn: {
		game: "Jogo",
		engine: "Motor",
		date: "Data",
	},

	modsPage: {
		openLocalModsFolderButton: "Abrir pasta de mods locais",
		openLoadlModsFolderTooltip:
			"Podes colocar mods nesta pasta com um rai-pal-manifest.json para os carregar diretamente sem passar pela base de dados online.",
		tableColumnMod: "Mod",
		tableColumnVersion: "Versão",
		tableColumnGameEngine: "Motor",
		tableColumnUnityBackend: "Backend",
		modByAuthor: "por {authorName}",
		modDeprecated: "Obsoleto",
		modDeprecatedTooltip:
			"Este mod está obsoleto. Deves desinstalá-lo e instalar uma alternativa mais recente.",
	},

	modModal: {
		runMod: "Executar",
		openModFolder: "Abrir pasta do mod",
		updateMod: "Atualizar mod",
		downloadMod: "Descarregar mod",
		deleteMod: "Eliminar mod",
		deleteModConfirmation:
			"Tens a certeza? Todos os ficheiros dentro da pasta do mod vão ser apagados.",
		byAuthor: "por {authorName}",
	},

	appDropdownMenu: {
		showGameThumbnails: "Mostrar imagens na lista de jogos",
		language: "Idioma",
		autoDetectedLanguage: "Auto-detetado - {languageName}",
		openLogsFolderButton: "Abrir Pasta de Registos",
		resetRaiPalSettingsButton: "Repor definições do Rai Pal",
		resetRaiPalSettingsTooltip:
			"Repõe filtros, diálogos de confirmação e provavelmente mais algumas coisas.",
		clearRaiPalCacheOpenModal: "Limpar cache do Rai Pal...",
		clearRaiPalCacheTooltip:
			"Limpa a cache da lista de jogos usada pelo Rai Pal.",
	},

	userMenu: {
		unknownUser: "Utilizador desconhecido",
		logOut: "Terminar sessão",
		signInWithDiscord: "Iniciar sessão com o Discord",
		discordAccessNote:
			"Os mods podem usar isto para aceder ao teu nome de utilizador do Discord, avatar, funções, etc.",
	},

	subPage: {
		back: "Voltar",
	},

	downloadStatusMenu: {
		clear: "Limpar concluídos",
	},

	steamCache: {
		resetSteamCacheButton: "Repor cache da Steam",
		resetSteamCacheModalTitle: "Repor cache da Steam",
		resetSteamCacheDescription:
			"Usa isto se o Rai Pal estiver a mostrar jogos que não possuis na Steam. Isto vai repor a cache da Steam e vais ter de a reiniciar. Vais receber um erro se o ficheiro já tiver sido apagado.",
		resetSteamCacheSuccess:
			"O ficheiro de cache foi apagado. Reinicia a Steam, espera alguns segundos e depois clica no botão de atualização no Rai Pal.",
	},

	steamShortcut: {
		addRaiPalSteamShortcutButton: "Adicionar Rai Pal à biblioteca da Steam",
		addRaiPalSteamShortcutModalTitle: "Adicionar Rai Pal à biblioteca da Steam",
		addRaiPalSteamShortcutDescription:
			"Isto é especialmente útil no Steam Deck, para poderes iniciar o Rai Pal no Modo Jogo. Vais ter de reiniciar a Steam depois de fazer isto.",
		addRaiPalSteamShortcutSuccess:
			"O Rai Pal foi adicionado à tua biblioteca da Steam. Reinicia a Steam para o veres.",
	},

	globalWineOverrides: {
		setUpEnvironmentButton: "Configurar ambiente Linux para o BepInEx",
		setUpEnvironmentTitle: "Configurar ambiente Linux para o BepInEx",
		setUpEnvironmentDescription:
			"Ao usar Proton/Wine no Linux, o BepInEx não será carregado automaticamente a menos que algumas definições do Wine estejam configuradas. Isto irá definir a variável de ambiente WINEDLLOVERRIDES para 'winhttp.dll=n,b' globalmente. Se preferires fazer isso manualmente, não cliques neste botão.",
		setUpEnvironmentSuccess:
			"O ficheiro foi escrito. Precisas de terminar sessão e voltar a iniciar, ou reiniciar o computador, para as alterações fazerem efeito.",
	},

	debugData: {
		debugDataTitle: "Dados de depuração",
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
		dontAskAgain: "Não voltar a perguntar",
	},

	urlModSources: {
		title: "Fontes de mods",
		add: "Adicionar fonte",
		addSourceDescription:
			"O Rai Pal irá verificar mods em todas as bases de dados fornecidas aqui, além da base de dados predefinida. Podes partilhar fontes de mods através do deep-link {deepLink}.",
		confirmModalTitle: "Confirmar fonte de mods",
		modsFound: "{count} mods encontrados nesta fonte",
		addSource: "Adicionar fonte",
		cancel: "Cancelar",
		loading: "A carregar mods...",
	},
};
