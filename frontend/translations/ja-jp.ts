export const jaJp = {
	tab: {
		games: "ゲーム",
		mods: "Mod",
		tools: "ツール",
		thanks: "感謝",
	},

	addGame: {
		button: "ゲームを追加",
		title: "ゲームを追加",
		dropField:
			"ここにゲームの実行ファイルをドラッグ＆ドロップするか、クリックしてファイルを選択してください。",
		note: "注: Rai Pal のウィンドウ内のどこにでもゲームの実行ファイルをドロップすると、このダイアログを開かずにインストール済みゲームリストに追加できます。",
	},

	refresh: {
		button: "更新",
		loading: "{items} を読み込み中...",
	},

	filterMenu: {
		button: "フィルター",
		resetButton: "リセット",
		searchPlaceholder: "検索...",
	},

	filterProperty: {
		provider: "プロバイダー",
		tags: "タグ",
		architecture: "アーキテクチャ",
		unityScriptingBackend: "Unity バックエンド",
		engine: "エンジン",
		status: "ステータス",
	},

	filterValue: {
		unknown: "不明",
		arch64: "64ビット",
		arch32: "32ビット",
		engineGodot: "Godot",
		engineGameMaker: "GameMaker",
		engineUnity: "Unity",
		engineUnreal: "Unreal",
		unityBackendIl2Cpp: "IL2CPP",
		unityBackendMono: "Mono",
		tagDemo: "デモ",
		tagVr: "ネイティブVR",
		tagUntagged: "タグなし",
		statusInstalled: "インストール済み",
		statusNotInstalled: "未インストール",
		providerSteam: "Steam",
		providerGog: "GOG",
		providerEpic: "Epic",
		providerItch: "itch.io",
		providerOrigin: "Origin",
		providerManual: "手動",
		providerXbox: "Xbox",
		providerEa: "EA",
		providerUbisoft: "Ubisoft",
	},

	filterValueNote: {
		providerXboxOnlyInstalled:
			"Xbox PCゲームはインストールされている場合のみ Rai Pal に表示されます。",
		engineGodotNotFullySupported:
			"Godot ゲームはまだ完全にはサポートされていません。",
		engineGameMakerNotFullySupported:
			"GameMaker ゲームはまだ完全にはサポートされていません。",
	},

	providerCommand: {
		installGame: "インストール",
		showGameInLibrary: "ライブラリで表示",
		showGameInStore: "ストアページを開く",
		startGame: "ゲームを開始",
		openGamePageInBrowser: "ブラウザで開く",
	},

	gameModal: {
		startGameButton: "ゲームを開始",
		startGameExecutable: "実行ファイルで開始",
		startGameViaProvider: "{provider} 経由で開始",
		foldersDropdown: "フォルダー",
		openGameFilesFolder: "ゲームファイルのフォルダーを開く",
		openInstalledModsFolder: "インストール済み Mod フォルダーを開く",
		removeFromRaiPal: "Rai Pal から削除",
		removeGameConfirmation:
			"このゲームを Rai Pal から削除してもよろしいですか？",
		refreshGame: "更新",
		failedToReadGameInfo:
			"このゲームの重要な情報を読み取れませんでした。実行ファイルが保護されている可能性があります。一部の Mod がインストールできない場合があります。",
		failedToDetermineEngine:
			"このゲームのエンジンを特定できませんでした。一部の Mod がインストールできない場合があります。",
		gameModsLabel: "Mod",
		gameNotInstalledWarning:
			"このゲームはインストールされていないため、互換性のある Mod を正確に判定できません。インストールすれば、より正確な情報を表示できます。",
		uninstallAllModsButton: "すべての Mod をアンインストール",
		uninstallAllModsConfirmation:
			"本当に削除しますか？このゲームの Mod フォルダー内のすべてのファイルが削除されますが、ゲーム本体のファイルは削除されません。",
	},

	gamesTableColumn: {
		game: "ゲーム",
		engine: "エンジン",
		date: "日付",
	},

	modsPage: {
		openModsFolderButton: "Mod フォルダーを開く",
		tableColumnMod: "Mod",
		tableColumnVersion: "バージョン",
		tableColumnModLoader: "ローダー",
		tableColumnGameEngine: "エンジン",
		tableColumnUnityBackend: "バックエンド",
		modByAuthor: "{authorName} 作",
		modDeprecated: "非推奨",
		modDeprecatedTooltip:
			"この Mod は非推奨です。アンインストールして、新しい代替 Mod を使用してください。",
		modOutdated: "Mod が古い",
	},

	modModal: {
		runMod: "実行",
		openModFolder: "Mod フォルダーを開く",
		updateMod: "Mod を更新",
		downloadMod: "Mod をダウンロード",
		deleteMod: "Mod を削除",
		deleteModConfirmation:
			"本当に削除しますか？Mod フォルダー内のすべてのファイルが失われます。",
		byAuthor: "{authorName} 作",
	},

	commandButton: {
		cancel: "キャンセル",
		dontAskAgain: "次回から確認しない",
	},
} as const;
