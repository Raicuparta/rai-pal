import { Localization } from "./localizations";

export const frFr: Localization = {
	meta: {
		nativeName: "Français (France)",
	},

	tab: {
		games: "Jeux",
		mods: "Mods",
		thanks: "Remerciements",
	},

	gamesPage: {
		emptyGamesList:
			"Rai Pal n'a trouvé aucun jeu du tout. Rappelez-vous que Rai Pal doit trouver des applications installées d'autres fournisseurs de jeux, comme Steam, Epic, etc.",
		emptyFilteredGamesList:
			"Rien ! Tous vos jeux sont cachés à cause des filtres que vous avez sélectionnés. Effacez vos filtres pour revoir vos beaux jeux.",
		emptyGamesLoading: "Recherche de vos jeux...",
	},

	addGame: {
		button: "Ajouter un jeu",
		title: "Ajouter un jeu",
		dropField:
			"Glissez et déposez un exécutable de jeu ici, ou cliquez pour sélectionner un fichier.",
		note: "Remarque : vous pouvez déposer des fichiers exécutables de jeu n'importe où dans la fenêtre de Rai Pal pour les ajouter à la liste des jeux installés sans ouvrir ce dialogue.",
	},

	refresh: {
		button: "Rafraîchir",
		buttonUpdateRemoteDatabases: "Mettre à jour les bases de données distantes",
		loading: "Chargement de {items}...",
	},

	filterMenu: {
		button: "Filtrer",
		resetButton: "Réinitialiser",
		searchPlaceholder: "Rechercher...",
	},

	filterProperty: {
		provider: "Fournisseur",
		tags: "Tag",
		architecture: "Architecture",
		unityBackend: "Backend Unity",
		engine: "Moteur",
		status: "Statut",
	},

	filterValue: {
		unknown: "Inconnu",
		arch64: "64 bits",
		arch32: "32 bits",
		tagDemo: "Démo",
		tagVr: "VR native",
		tagUntagged: "Non tagué",
		statusInstalled: "Installé",
		statusNotInstalled: "Non installé",
		providerManual: "Manuel",
	},

	filterValueNote: {
		providerXboxOnlyInstalled:
			"Affiche uniquement les jeux Xbox PC s'ils sont installés.",
		engineGodotNotFullySupported:
			"Les jeux Godot ne sont pas encore entièrement pris en charge.",
		engineGameMakerNotFullySupported:
			"Les jeux GameMaker ne sont pas encore entièrement pris en charge.",
	},

	providerCommand: {
		installGame: "Installer",
		showGameInLibrary: "Afficher dans la bibliothèque",
		showGameInStore: "Ouvrir la page du magasin",
		startGameViaProvider: "Lancer le jeu",
		startGameViaExe: "Exécuter l'exécutable du jeu",
		openGamePageInBrowser: "Ouvrir dans le navigateur",
	},

	gameModal: {
		startGameButton: "Lancer le jeu",
		startGameExecutable: "Lancer l'exécutable du jeu",
		startGameViaProvider: "Lancer le jeu via {provider}",
		foldersDropdown: "Dossiers",
		openGameFilesFolder: "Ouvrir le dossier des fichiers du jeu",
		openInstalledModsFolder: "Ouvrir le dossier des mods installés",
		removeFromRaiPal: "Retirer de Rai Pal",
		removeGameConfirmation:
			"Êtes-vous sûr de vouloir retirer ce jeu de Rai Pal ?",
		refreshGame: "Rafraîchir",
		failedToReadGameInfo:
			"Impossible de lire certaines informations importantes sur ce jeu. Cela peut être dû à la protection de l'exécutable. Certains mods peuvent échouer à s'installer.",
		failedToDetermineEngine:
			"Impossible de déterminer le moteur de ce jeu. Certains mods peuvent échouer à s'installer.",
		gameModsLabel: "Mods",
		gameNotInstalledWarning:
			"Ce jeu n'est pas installé, donc je ne suis pas sûr à 100% de quels mods sont compatibles. Ceux que vous voyez ci-dessous pourraient fonctionner. Si vous installez le jeu, je vous montrerai des informations plus précises.",
		uninstallAllModsButton: "Désinstaller tous les mods",
		uninstallAllModsConfirmation:
			"Êtes-vous sûr ? Cela supprimera tous les fichiers dans le dossier des mods de ce jeu. Cela ne supprimera aucun fichier du jeu lui-même.",
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

	appDropdownMenu: {
		showGameThumbnails: "Afficher les vignettes des jeux dans la liste",
		language: "Langue",
		autoDetectedLanguage: "Détecté automatiquement - {languageName}",
		resetRaiPalSettingsButton: "Réinitialiser les paramètres de Rai Pal",
		resetRaiPalSettingsTooltip:
			"Réinitialisera les filtres, les dialogues de confirmation, probablement d'autres choses.",
		openLogsFolderButton: "Ouvrir le dossier des journaux",
		clearRaiPalCacheOpenModal: "Effacer le cache de Rai Pal",
		clearRaiPalCacheTooltip:
			"Efface le cache de la liste des jeux utilisé par Rai Pal.",
	},

	steamCache: {
		resetSteamCacheButtonOpenModal: "Réinitialiser le cache Steam...",
		resetSteamCacheModalTitle: "Réinitialiser le cache Steam",
		resetSteamCacheDescription:
			"Utilisez ceci si Rai Pal affiche des jeux que vous ne possédez pas réellement sur Steam. Cela réinitialisera le cache de Steam, puis vous devrez redémarrer Steam. Vous obtiendrez une erreur si le fichier a déjà été supprimé.",
		resetSteamCacheSuccess:
			"Le fichier de cache a été supprimé. Veuillez redémarrer Steam, attendre quelques secondes, puis appuyer sur le bouton de rafraîchissement de Rai Pal.",
	},

	debugData: {
		debugDataTitle: "Données de débogage",
		debugDataCopy: "Copier les données de débogage",
	},

	thanksPage: {
		intro:
			"Bonjour. J'ai créé Rai Pal. J'ai également créé d'autres mods VR dans le passé, et je travaille actuellement sur un mod VR universel pour les jeux Unity. Si vous aimez ce que je fais et que vous souhaitez en voir plus, veuillez envisager de faire un don ! Vous pouvez également me soutenir en achetant un de mes mods gratuits sur itch.io.",
		starRaiPalOnGitHub: "Étoile Rai Pal sur GitHub",
		otherModdersTitle: "Autres moddeurs",
		otherModdersDescription:
			"Rai Pal est conçu pour vous aider à gérer le modding de jeux, et nous ne pouvons pas le faire sans les outils créés par d'autres développeurs. Certaines de ces personnes n'ont pas de liens de donation, mais je suis extrêmement reconnaissant pour leur travail.",
		modderOnWebsite: "{modderName} sur {website}",
		patreonLeaderboard: "Classement Patreon",
		rankedByPatreonDonationAmount: "Classé par montant total des dons à vie.",
		patreonProfilePrivateNotice:
			"Si vous ne vous voyez pas ici, c'est parce que votre profil Patreon est privé.",
	},

	commandButton: {
		cancel: "Annuler",
		dontAskAgain: "Ne plus demander",
	},
} as const;
