import { Localization } from "./localizations";

export const esEs: Localization = {
	meta: {
		nativeName: "Español (España)",
	},
	tab: {
		games: "Juegos",
		mods: "Mods",
		tools: "Herramientas",
		thanks: "Agradecimientos",
	},

	addGame: {
		button: "Añadir juego",
		title: "Añadir juego",
		dropField:
			"Arrastra y suelta un ejecutable de juego aquí, o haz clic para seleccionar un archivo.",
		note: "Nota: puedes soltar archivos ejecutables de juegos en cualquier parte de la ventana de Rai Pal para añadirlos a la lista de juegos instalados sin abrir este diálogo.",
	},

	refresh: {
		button: "Actualizar",
		loading: "Cargando {items}...",
	},

	filterMenu: {
		button: "Filtrar",
		resetButton: "Restablecer",
		searchPlaceholder: "Buscar...",
	},

	filterProperty: {
		provider: "Proveedor",
		tags: "Etiqueta",
		architecture: "Arquitectura",
		unityScriptingBackend: "Backend de Unity",
		engine: "Motor",
		status: "Estado",
	},

	filterValue: {
		unknown: "Desconocido",
		arch64: "64 bits",
		arch32: "32 bits",
		tagDemo: "Demo",
		tagVr: "VR Nativo",
		tagUntagged: "Sin etiqueta",
		statusInstalled: "Instalado",
		statusNotInstalled: "No instalado",
		providerManual: "Manual",
	},

	filterValueNote: {
		providerXboxOnlyInstalled:
			"Los juegos de Xbox para PC solo aparecen en Rai Pal cuando están instalados.",
		engineGodotNotFullySupported:
			"Los juegos de Godot no están totalmente soportados aún.",
		engineGameMakerNotFullySupported:
			"Los juegos de GameMaker no están totalmente soportados aún.",
	},

	providerCommand: {
		installGame: "Instalar",
		showGameInLibrary: "Mostrar en Biblioteca",
		showGameInStore: "Abrir página de tienda",
		startGame: "Iniciar juego",
		openGamePageInBrowser: "Abrir en navegador",
	},

	gameModal: {
		startGameButton: "Iniciar juego",
		startGameExecutable: "Iniciar ejecutable del juego",
		startGameViaProvider: "Iniciar juego vía {provider}",
		foldersDropdown: "Carpetas",
		openGameFilesFolder: "Abrir carpeta de archivos del juego",
		openInstalledModsFolder: "Abrir carpeta de mods instalados",
		removeFromRaiPal: "Eliminar de Rai Pal",
		removeGameConfirmation:
			"¿Seguro que quieres eliminar este juego de Rai Pal?",
		refreshGame: "Actualizar",
		failedToReadGameInfo:
			"No se pudo leer información importante sobre este juego. Podría deberse a que el ejecutable está protegido. Algunos mods podrían fallar al instalarse.",
		failedToDetermineEngine:
			"No se pudo determinar el motor de este juego. Algunos mods podrían fallar al instalarse.",
		gameModsLabel: "Mods",
		gameNotInstalledWarning:
			"Este juego no está instalado, por lo que no estoy 100% seguro de qué mods son compatibles. Los que ves abajo podrían funcionar. Si instalas el juego, te mostraré información más precisa.",
		uninstallAllModsButton: "Desinstalar todos los mods",
		uninstallAllModsConfirmation:
			"¿Seguro? Esto eliminará todos los archivos en la carpeta de mods de este juego. No eliminará ningún archivo del juego en sí.",
	},

	gamesTableColumn: {
		game: "Juego",
		engine: "Motor",
		date: "Fecha",
	},

	modsPage: {
		openModsFolderButton: "Abrir carpeta de mods",
		tableColumnMod: "Mod",
		tableColumnVersion: "Versión",
		tableColumnModLoader: "Cargador",
		tableColumnGameEngine: "Motor",
		tableColumnUnityBackend: "Backend",
		modByAuthor: "por {authorName}",
		modDeprecated: "Obsoleto",
		modDeprecatedTooltip:
			"Este mod está obsoleto. Deberías desinstalarlo e instalar una alternativa más reciente.",
		modOutdated: "Mod desactualizado",
	},

	modModal: {
		runMod: "Ejecutar",
		openModFolder: "Abrir carpeta del mod",
		updateMod: "Actualizar mod",
		downloadMod: "Descargar mod",
		deleteMod: "Eliminar mod",
		deleteModConfirmation:
			"¿Seguro? Se perderán todos los archivos dentro de la carpeta del mod.",
		byAuthor: "por {authorName}",
	},

	toolsPage: {
		openLogsFolderButton: "Abrir carpeta de registros",
		resetRaiPalSettingsButton: "Restablecer configuración de Rai Pal",
		resetRaiPalSettingsTooltip:
			"Restablecerá filtros, diálogos de confirmación y probablemente otras cosas.",
		clearCacheButton: "Limpiar caché de Rai Pal",
		clearCacheTooltip:
			"Limpia la caché de la lista de juegos usada por Rai Pal.",
	},

	appSettings: {
		showGameThumbnails: "Mostrar imágenes en la lista de juegos",
		language: "Idioma",
		autoDetectedLanguage: "Auto-detectado - {languageName}",
	},

	steamCache: {
		resetSteamCacheButton: "Restablecer caché de Steam",
		resetSteamCacheModalTitle: "Restablecer caché de Steam",
		resetSteamCacheDescription:
			"Usa esto si Rai Pal muestra juegos que realmente no posees en Steam. Esto restablecerá la caché de Steam y luego tendrás que reiniciar Steam. Recibirás un error si el archivo ya ha sido eliminado.",
		resetSteamCacheSuccess:
			"El archivo de caché ha sido eliminado. Reinicia Steam, espera unos segundos y luego presiona el botón de actualización en Rai Pal.",
	},

	debugData: {
		debugDataTitle: "Datos de depuración",
		debugDataCopy: "Copiar datos de depuración",
	},

	thanksPage: {
		intro:
			"Hola. Hice Rai Pal. También hice otros mods de VR en el pasado y actualmente estoy trabajando en un mod de VR universal para juegos de Unity. Si te gusta lo que hago y quieres ver más, ¡considera hacer una donación! También puedes apoyarme comprando uno de mis mods gratuitos en itch.io.",
		starRaiPalOnGitHub: "Dale una estrella a Rai Pal en GitHub",
		otherModdersTitle: "Otros modders",
		otherModdersDescription:
			"Rai Pal está diseñado para ayudarte a gestionar mods de juegos, y no podríamos hacerlo sin las herramientas que otros desarrolladores han creado. Algunos de estos desarrolladores no tienen enlaces de donación, pero estoy muy agradecido por su trabajo.",
		modderOnWebsite: "{modderName} en {website}",
		patreonLeaderboard: "Tabla de líderes de Patreon",
		rankedByPatreonDonationAmount:
			"Clasificado por la cantidad total de donaciones de por vida.",
		patreonProfilePrivateNotice:
			"Si no te ves aquí, es porque tu perfil de Patreon es privado.",
	},

	commandButton: {
		cancel: "Cancelar",
		dontAskAgain: "No preguntar de nuevo",
	},
};
