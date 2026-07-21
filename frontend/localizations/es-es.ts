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
		directoryButton: "Scan a folder recursively for games. Can be slow!",
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
		mod: "Mod",
	},

	filterValue: {
		unknown: "Desconocido",
		arch64: "64-bit",
		arch32: "32-bit",
		tagDemo: "Demo",
		tagVr: "VR nativa",
		tagUntagged: "Sin etiqueta",
		statusInstalled: "Instalado",
		statusNotInstalled: "No instalado",
		providerManual: "Manual",
	},

	filterValueNote: {
		providerXboxOnlyInstalled:
			"Solo muestra juegos de Xbox para PC si están instalados.",
		engineGodotNotFullySupported:
			"Los juegos de Godot no son totalmente compatibles aún.",
		engineGameMakerNotFullySupported:
			"Los juegos de GameMaker no son totalmente compatibles aún.",
	},

	providerCommand: {
		installGame: "Instalar",
		showGameInLibrary: "Mostrar en la biblioteca",
		showGameInStore: "Abrir la página de la tienda",
		startGameViaProvider: "Iniciar juego",
		startGameViaExe: "Abrir ejecutable del juego",
		openGamePageInBrowser: "Abrir en el navegador",
	},

	gameModal: {
		startGameButton: "Iniciar juego",
		startGameExecutable: "Iniciar ejecutable del juego",
		startGameViaProvider: "Iniciar juego vía {provider}",
		foldersDropdown: "Carpetas",
		openGameFilesFolder: "Abrir carpeta de archivos del juego",
		openInstalledModsFolder: "Abrir carpeta de mods instalados",
		openGameDataFolder: "Abrir carpeta de datos de la aplicación del juego",
		openGameWinePrefixFolder: "Abrir carpeta del prefijo Wine del juego",
		openGameWineBinaryFolder: "Abrir carpeta del binario Wine del juego",
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
			"¿Seguro? Esto borrará todos los archivos de la carpeta de mods del juego. Eso sí, los archivos del juego no se tocan.",

		incompatibleGameModsLabel: "Mods Incompatibles",

		incompatibleGameModsDescription:
			"Los mods listados aquí no se pueden instalar porque no son compatibles con la versión del motor de este juego.",

		otherThings: "Otras cosas",

		otherThingsDescription:
			"Estas son principalmente dependencias y otras cosas que normalmente no necesitas modificar directamente.",
	},

	gameModRow: {
		editModConfig: "Editar configuración del mod",
		openModConfigFolderTooltip:
			"Abrir la carpeta con los archivos de configuración del mod",
		openModFolder: "Abrir carpeta del mod",
		updateMod: "Actualizar",
		installMod: "Instalar",
		installModAnticheatWarning:
			"Atención: ¡Ten cuidado al instalar mods en juegos multijugador! El anticheat puede detectar algunos mods y banearte, incluso si los mods parecen inofensivos.",
		reinstallMod: "Reinstalar",
		uninstallMod: "Desinstalar",
		runMod: "Ejecutar",
		downloadRemoteConfig: "Descargar configuración recomendada",
		remoteConfigAvailable:
			"Configuración recomendada disponible. Se descargará si aún no tienes una configuración. También puedes forzar la descarga desde el menú de tres puntos.",
		modOutdated: "Mod desactualizado",
		cantUninstallModWithDependants:
			"No se puede desinstalar un mod que tiene dependientes. Desinstala primero los mods que dependen de este.",
	},

	gamesTableColumn: {
		game: "Juego",
		engine: "Motor",
		date: "Fecha",
	},

	modsPage: {
		openLocalModsFolderButton: "Abrir carpeta de mods locales",
		openLoadlModsFolderTooltip:
			"Puedes poner mods en esta carpeta con un rai-pal-manifest.json para cargarlos directamente sin pasar por la base de datos en línea.",
		tableColumnMod: "Mod",
		tableColumnVersion: "Versión",
		tableColumnGameEngine: "Motor",
		tableColumnUnityBackend: "Backend",
		modByAuthor: "por {authorName}",
		modDeprecated: "Obsoleto",
		modDeprecatedTooltip:
			"Este mod está obsoleto. Deberías desinstalarlo e instalar una alternativa más reciente.",
	},

	modModal: {
		runMod: "Ejecutar",
		openModFolder: "Abrir carpeta del mod",
		updateMod: "Actualizar mod",
		downloadMod: "Descargar mod",
		deleteMod: "Eliminar mod",
		deleteModConfirmation:
			"¿Seguro? Se borrarán todos los archivos de la carpeta del mod.",
		byAuthor: "por {authorName}",
	},

	appDropdownMenu: {
		showGameThumbnails: "Mostrar miniaturas de juegos en la lista",
		language: "Idioma",
		autoDetectedLanguage: "Auto-detectado - {languageName}",
		resetRaiPalSettingsButton: "Restablecer configuración de Rai Pal",
		resetRaiPalSettingsTooltip:
			"Restablecerá filtros, diálogos de confirmación y demás.",
		openLogsFolderButton: "Abrir carpeta de registros",
		clearRaiPalCacheOpenModal: "Borrar caché de Rai Pal",
		clearRaiPalCacheTooltip:
			"Borra la caché de la lista de juegos que usa Rai Pal.",
	},

	userMenu: {
		unknownUser: "Usuario desconocido",
		logOut: "Cerrar sesión",
		signInWithDiscord: "Iniciar sesión con Discord",
		discordAccessNote:
			"Los mods pueden usar esto para acceder a tu nombre de usuario de Discord, avatar, roles, etc.",
	},

	subPage: {
		back: "Volver",
	},

	downloadStatusMenu: {
		clear: "Quitar completados",
	},

	steamCache: {
		resetSteamCacheButton: "Restablecer caché de Steam",
		resetSteamCacheModalTitle: "Restablecer caché de Steam",
		resetSteamCacheDescription:
			"Usa esto si Rai Pal está mostrando juegos que no posees realmente en Steam. Esto restablecerá la caché de Steam, y luego tendrás que reiniciar Steam. Recibirás un error si el archivo ya ha sido eliminado.",
		resetSteamCacheSuccess:
			"El archivo de caché ha sido eliminado. Por favor, reinicia Steam, espera unos segundos y luego presiona el botón de actualizar en Rai Pal.",
	},

	steamShortcut: {
		addRaiPalSteamShortcutButton: "Agregar Rai Pal a la biblioteca de Steam",
		addRaiPalSteamShortcutModalTitle:
			"Agregar Rai Pal a la biblioteca de Steam",
		addRaiPalSteamShortcutDescription:
			"Esto es especialmente útil en Steam Deck para poder iniciar Rai Pal en Modo Juego. Tendrás que reiniciar Steam después de hacerlo.",
		addRaiPalSteamShortcutSuccess:
			"Rai Pal se ha agregado a tu biblioteca de Steam. Reinicia Steam para verlo.",
	},

	globalWineOverrides: {
		setUpEnvironmentButton: "Configurar entorno Linux para BepInEx",
		setUpEnvironmentTitle: "Configurar entorno Linux para BepInEx",
		setUpEnvironmentDescription:
			"Al usar Proton/Wine en Linux, BepInEx no se cargará automáticamente a menos que se configuren algunos ajustes de Wine. Esto establecerá la variable de entorno WINEDLLOVERRIDES en 'winhttp.dll=n,b' globalmente. Si prefieres hacerlo manualmente, no hagas clic en este botón.",
		setUpEnvironmentSuccess:
			"El archivo se ha creado. Tendrás que cerrar sesión y volver a iniciarla, o reiniciar el ordenador, para que los cambios se apliquen.",
	},

	debugData: {
		debugDataTitle: "Datos de depuración",
		debugDataCopy: "Copiar datos de depuración",
	},

	thanksPage: {
		intro:
			"Hola. Hice Rai Pal. También hice otros mods de VR en el pasado, y actualmente estoy trabajando en un mod universal de VR para juegos de Unity. Si te gusta lo que hago y te gustaría ver más, anímate a donar. También puedes apoyarme comprando uno de mis mods gratuitos en itch.io.",
		starRaiPalOnGitHub: "Dale una estrella a Rai Pal en GitHub",
		otherModdersTitle: "Otros modders",
		otherModdersDescription:
			"Rai Pal está pensado para ayudarte a gestionar mods, y no podemos hacerlo sin las herramientas que otros desarrolladores han creado. Algunas de estas personas no tienen enlaces de donación, pero estoy muy agradecido por su trabajo.",
		modderOnWebsite: "{modderName} en {website}",
		patreonLeaderboard: "Ranking de Patreon",
		rankedByPatreonDonationAmount:
			"Ordenado por el total de donaciones acumuladas.",
		patreonProfilePrivateNotice:
			"Si no te ves aquí, es porque tu perfil de Patreon es privado.",
	},

	commandButton: {
		cancel: "Cancelar",
		dontAskAgain: "No preguntar de nuevo",
	},

	urlModSources: {
		title: "Fuentes de mods",
		add: "Añadir fuente",
		addSourceDescription:
			"Rai Pal buscará mods en todas las bases de datos proporcionadas aquí, además de la base de datos predeterminada. Puedes compartir fuentes de mods mediante el enlace profundo {deepLink}.",
		confirmModalTitle: "Confirmar fuente de mods",
		modsFound: "{count} mods encontrados en esta fuente",
		addSource: "Añadir fuente",
		cancel: "Cancelar",
		loading: "Cargando mods...",
	},
} as const;
