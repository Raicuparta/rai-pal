import { Translation } from "./translations";

export const frFr: Translation = {
	tab: {
		games: "Jeux",
		mods: "Mods",
		tools: "Outils",
		thanks: "Remerciements",
	},

	addGame: {
		button: "Ajouter un jeu",
		title: "Ajouter un jeu",
		dropField:
			"Glissez-déposez un exécutable de jeu ici, ou cliquez pour sélectionner un fichier.",
		note: "Remarque : vous pouvez déposer des fichiers exécutables du jeu n'importe où dans la fenêtre de Rai Pal pour les ajouter à la liste des jeux installés sans ouvrir ce dialogue.",
	},

	refresh: {
		button: "Rafraîchir",
		loading: "Chargement de {items}...",
	},

	filterMenu: {
		button: "Filtrer",
		resetButton: "Réinitialiser",
		searchPlaceholder: "Rechercher...",
	},

	filterProperty: {
		provider: "Fournisseur",
		tags: "Étiquette",
		architecture: "Architecture",
		unityScriptingBackend: "Backend Unity",
		engine: "Moteur",
		status: "Statut",
	},

	filterValue: {
		unknown: "Inconnu",
		arch64: "64 bits",
		arch32: "32 bits",
		tagDemo: "Démo",
		tagVr: "VR native",
		tagUntagged: "Non étiqueté",
		statusInstalled: "Installé",
		statusNotInstalled: "Non installé",
		providerManual: "Manuel",
	},

	filterValueNote: {
		providerXboxOnlyInstalled:
			"Les jeux Xbox PC s'affichent uniquement dans Rai Pal une fois installés.",
		engineGodotNotFullySupported:
			"Les jeux Godot ne sont pas encore entièrement pris en charge.",
		engineGameMakerNotFullySupported:
			"Les jeux GameMaker ne sont pas encore entièrement pris en charge.",
	},

	providerCommand: {
		installGame: "Installer",
		showGameInLibrary: "Afficher dans la bibliothèque",
		showGameInStore: "Ouvrir la page de la boutique",
		startGame: "Démarrer le jeu",
		openGamePageInBrowser: "Ouvrir dans le navigateur",
	},

	gameModal: {
		startGameButton: "Démarrer le jeu",
		startGameExecutable: "Démarrer l'exécutable du jeu",
		startGameViaProvider: "Démarrer le jeu via {provider}",
		foldersDropdown: "Dossiers",
		openGameFilesFolder: "Ouvrir le dossier des fichiers du jeu",
		openInstalledModsFolder: "Ouvrir le dossier des mods installés",
		removeFromRaiPal: "Retirer de Rai Pal",
		removeGameConfirmation:
			"Êtes-vous sûr de vouloir retirer ce jeu de Rai Pal ?",
		refreshGame: "Rafraîchir",
		failedToReadGameInfo:
			"Impossible de lire certaines informations importantes sur ce jeu. Cela peut être dû à la protection de l'exécutable. Certains mods pourraient échouer à s'installer.",
		failedToDetermineEngine:
			"Impossible de déterminer le moteur de ce jeu. Certains mods pourraient ne pas s'installer.",
		gameModsLabel: "Mods",
		gameNotInstalledWarning:
			"Ce jeu n'est pas installé, donc je ne suis pas sûr à 100% de la compatibilité des mods. Ceux que vous voyez ci-dessous pourraient fonctionner. Si vous installez le jeu, je vous fournirai des informations plus précises.",
		uninstallAllModsButton: "Désinstaller tous les mods",
		uninstallAllModsConfirmation:
			"Êtes-vous sûr ? Cela supprimera tous les fichiers dans le dossier des mods de ce jeu. Cela ne supprimera pas les fichiers du jeu lui-même.",
	},

	gamesTableColumn: {
		game: "Jeu",
		engine: "Moteur",
		date: "Date",
	},

	modsPage: {
		openModsFolderButton: "Ouvrir le dossier des mods",
		tableColumnMod: "Mod",
		tableColumnVersion: "Version",
		tableColumnModLoader: "Chargeur",
		tableColumnGameEngine: "Moteur",
		tableColumnUnityBackend: "Backend",
		modByAuthor: "par {authorName}",
		modDeprecated: "Obsolète",
		modDeprecatedTooltip:
			"Ce mod est obsolète. Vous devriez le désinstaller et installer une alternative plus récente.",
		modOutdated: "Mod obsolète",
	},

	modModal: {
		runMod: "Exécuter",
		openModFolder: "Ouvrir le dossier du mod",
		updateMod: "Mettre à jour le mod",
		downloadMod: "Télécharger le mod",
		deleteMod: "Supprimer le mod",
		deleteModConfirmation:
			"Êtes-vous sûr ? Tous les fichiers dans le dossier du mod seront perdus.",
		byAuthor: "par {authorName}",
	},

	toolsPage: {
		openLogsFolderButton: "Ouvrir le dossier des journaux",
		resetRaiPalSettingsButton: "Réinitialiser les paramètres de Rai Pal",
		resetRaiPalSettingsTooltip:
			"Réinitialisera les filtres, les boîtes de dialogue de confirmation et sans doute d'autres paramètres.",
		clearCacheButton: "Vider le cache de Rai Pal",
		clearCacheTooltip: "Vide la liste de jeux mise en cache par Rai Pal.",
	},

	steamCache: {
		resetSteamCacheButton: "Réinitialiser le cache Steam",
		resetSteamCacheModalTitle: "Réinitialiser le cache Steam",
		resetSteamCacheDescription:
			"Utilisez ceci si Rai Pal affiche des jeux que vous ne possédez pas réellement sur Steam. Cela réinitialisera le cache Steam, et vous devrez redémarrer Steam. Vous obtiendrez une erreur si le fichier a déjà été supprimé.",
		resetSteamCacheSuccess:
			"Le fichier de cache a été supprimé. Redémarrez Steam, attendez quelques secondes, puis appuyez sur le bouton de rafraîchissement de Rai Pal.",
	},

	debugData: {
		debugDataTitle: "Données de débogage",
		debugDataCopy: "Copier les données de débogage",
	},

	thanksPage: {
		intro:
			"Bonjour. J'ai créé Rai Pal. J'ai aussi créé d'autres mods VR auparavant, et je travaille actuellement sur un mod VR universel pour les jeux Unity. Si vous appréciez mon travail et souhaitez en voir plus, merci de me soutenir ! Vous pouvez également m'aider en achetant l'un de mes mods gratuits sur itch.io.",
		starRaiPalOnGitHub: "Donnez une étoile à Rai Pal sur GitHub",
		otherModdersTitle: "Autres moddeurs",
		otherModdersDescription:
			"Rai Pal est conçu pour vous aider à gérer la modding de jeux, et nous ne pouvons pas le faire sans les outils créés par d'autres développeurs. Certains n'ont pas de liens de dons, mais je leur suis extrêmement reconnaissant pour leur travail.",
		modderOnWebsite: "{modderName} sur {website}",
		patreonLeaderboard: "Classement Patreon",
		rankedByPatreonDonationAmount:
			"Classé selon le montant total des dons à vie.",
		patreonProfilePrivateNotice:
			"Si vous ne vous voyez pas ici, c'est parce que votre profil Patreon est privé.",
	},

	commandButton: {
		cancel: "Annuler",
		dontAskAgain: "Ne plus demander",
	},
};
