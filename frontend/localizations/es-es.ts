import { Localization } from "./localizations";

export const esEs: Localization = {
	meta: {
		nativeName: "Español (España)",
	},

	tab: {
		games: "Juegos",
		mods: "Mods",
		thanks: "Agradecimientos",
	},

	gamesPage: {
		emptyGamesList:
			"Rai Pal no encontró ningún juego. Recuerda que Rai Pal necesita encontrar aplicaciones instaladas de otros proveedores de juegos, como Steam, Epic, etc.",
		emptyFilteredGamesList:
			"¡Nada! Todos tus juegos están ocultos debido a los filtros que seleccionaste. Limpia tus filtros para ver tus hermosos juegos de nuevo.",
		emptyGamesLoading: "Buscando tus juegos...",
	},

	addGame: {
		button: "Agregar juego",
		title: "Agregar juego",
		dropField:
			"Arrastra y suelta un ejecutable de juego aquí, o haz clic para seleccionar un archivo.",
		note: "Nota: puedes soltar archivos ejecutables de juegos en cualquier parte de la ventana de Rai Pal para agregarlos a la lista de juegos instalados sin abrir este diálogo.",
	},

	refresh: {
		button: "Actualizar",
		buttonUpdateRemoteDatabases: "Actualizar bases de datos remotas",
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
		unityBackend: "Backend de Unity",
		engine: "Motor",
		status: "Estado",
	},

	filterValue: {
		unknown: "Desconocido",
		arch64: "64-bit",
		arch32: "32-bit",
		tagDemo: "Demo",
		tagVr: "VR Nativo",
		tagUntagged: "Sin etiqueta",
		statusInstalled: "Instalado",
		statusNotInstalled: "No instalado",
		providerManual: "Manual",
	},

	filterValueNote: {
		providerXboxOnlyInstalled:
			"Solo muestra juegos de Xbox para PC si están instalados.",
		engineGodotNotFullySupported:
			"Los juegos de Godot no están totalmente soportados aún.",
		engineGameMakerNotFullySupported:
			"Los juegos de GameMaker no están totalmente soportados aún.",
	},

	providerCommand: {
		installGame: "Instalar",
		showGameInLibrary: "Mostrar en la biblioteca",
		showGameInStore: "Abrir página de la tienda",
		startGameViaProvider: "Iniciar juego",
		startGameViaExe: "Ejecutar ejecutable del juego",
		openGamePageInBrowser: "Abrir en el navegador",
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
			"¿Estás seguro de que quieres eliminar este juego de Rai Pal?",
		refreshGame: "Actualizar",
		failedToReadGameInfo:
			"No se pudo leer información importante sobre este juego. Esto podría deberse a que el ejecutable está protegido. Algunos mods podrían fallar al instalarse.",
		failedToDetermineEngine:
			"No se pudo determinar el motor de este juego. Algunos mods podrían fallar al instalarse.",
		gameModsLabel: "Mods",
		gameNotInstalledWarning:
			"Este juego no está instalado, así que no estoy 100% seguro de qué mods son compatibles. Los que ves a continuación podrían funcionar. Si instalas el juego, te mostraré información más precisa.",
		uninstallAllModsButton: "Desinstalar todos los mods",
		uninstallAllModsConfirmation:
			"¿Estás seguro? Esto eliminará todos los archivos en la carpeta de mods de este juego. Sin embargo, no eliminará ningún archivo del juego en sí.",

		incompatibleGameModsLabel: "Mods Incompatibles",

		incompatibleGameModsDescription:
			"Los mods listados aquí no se pueden instalar porque no son compatibles con la versión del motor de este juego.",
	},

	gameModRow: {
		editModConfig: "Editar Configuración del Mod",
		openModConfigFolderTooltip:
			"Abrir carpeta que contiene los archivos de configuración de este mod",
		openModFolder: "Abrir Carpeta del Mod",
		updateMod: "Actualizar",
		installMod: "Instalar",
		uninstallMod: "Desinstalar",
		runMod: "Ejecutar",
		downloadRemoteConfig: "Descargar Configuración Recomendada",
		remoteConfigAvailable:
			"Configuración recomendada disponible. Se descargará si aún no tienes una configuración. También puedes forzar la descarga desde el menú de tres puntos.",
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
			"¿Estás seguro? Se perderán todos los archivos dentro de la carpeta del mod.",
		byAuthor: "por {authorName}",
	},

	appDropdownMenu: {
		showGameThumbnails: "Mostrar miniaturas de juegos en la lista",
		language: "Idioma",
		autoDetectedLanguage: "Auto-detectado - {languageName}",
		resetRaiPalSettingsButton: "Restablecer configuración de Rai Pal",
		resetRaiPalSettingsTooltip:
			"Restablecerá filtros, diálogos de confirmación, probablemente otras cosas.",
		openLogsFolderButton: "Abrir carpeta de registros",
		clearRaiPalCacheOpenModal: "Borrar caché de Rai Pal",
		clearRaiPalCacheTooltip:
			"Borra la caché de la lista de juegos utilizada por Rai Pal.",
	},

	steamCache: {
		resetSteamCacheButton: "Restablecer caché de Steam",
		resetSteamCacheModalTitle: "Restablecer caché de Steam",
		resetSteamCacheDescription:
			"Usa esto si Rai Pal está mostrando juegos que no posees realmente en Steam. Esto restablecerá la caché de Steam, y luego tendrás que reiniciar Steam. Recibirás un error si el archivo ya ha sido eliminado.",
		resetSteamCacheSuccess:
			"El archivo de caché ha sido eliminado. Por favor, reinicia Steam, espera unos segundos y luego presiona el botón de actualizar en Rai Pal.",
	},

	debugData: {
		debugDataTitle: "Datos de depuración",
		debugDataCopy: "Copiar datos de depuración",
	},

	thanksPage: {
		intro:
			"Hola. Hice Rai Pal. También hice otros mods de VR en el pasado, y actualmente estoy trabajando en un mod universal de VR para juegos de Unity. Si te gusta lo que hago y te gustaría ver más, por favor considera donar. También puedes apoyarme comprando uno de mis mods gratuitos en itch.io.",
		starRaiPalOnGitHub: "Dale una estrella a Rai Pal en GitHub",
		otherModdersTitle: "Otros modders",
		otherModdersDescription:
			"Rai Pal está destinado a ayudarte a gestionar la modificación de juegos, y no podemos hacerlo sin las herramientas que otros desarrolladores han creado. Algunas de estas personas no tienen enlaces de donación, pero estoy extremadamente agradecido por su trabajo.",
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
} as const;
