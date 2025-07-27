import { Localization } from "./localizations";

export const jaJp: Localization = {
	meta: {
		nativeName: "日本語",
	},

	tab: {
		games: "ゲーム",
		mods: "Mod",
		thanks: "感謝",
	},

	gamesPage: {
		emptyGamesList:
			"Rai Palはゲームを全く見つけられませんでした。Rai PalはSteam、Epicなどの他のゲームプロバイダーからインストールされたアプリを見つける必要があることを覚えておいてください。",
		emptyFilteredGamesList:
			"何もありません！選択したフィルターのために、すべてのゲームが非表示になっています。フィルターをクリアして、美しいゲームを再び見てください。",
		emptyGamesLoading: "ゲームを検索中...",
	},

	addGame: {
		button: "ゲームを追加",
		title: "ゲームを追加",
		dropField:
			"ここにゲームの実行ファイルをドラッグ＆ドロップするか、クリックしてファイルを選択してください。",
		note: "注: Rai Palのウィンドウ内のどこにでもゲームの実行ファイルをドロップして、ダイアログを開かずにインストール済みゲームリストに追加できます。",
	},

	refresh: {
		button: "更新",
		buttonUpdateRemoteDatabases: "リモートデータベースを更新",
		loading: "{items}を読み込み中...",
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
		unityBackend: "Unity バックエンド",
		engine: "エンジン",
		status: "ステータス",
	},

	filterValue: {
		unknown: "不明",
		arch64: "64ビット",
		arch32: "32ビット",
		tagDemo: "デモ",
		tagVr: "ネイティブVR",
		tagUntagged: "タグなし",
		statusInstalled: "インストール済み",
		statusNotInstalled: "未インストール",
		providerManual: "手動",
	},

	filterValueNote: {
		providerXboxOnlyInstalled:
			"インストールされている場合のみPC Xboxゲームを表示します。",
		engineGodotNotFullySupported:
			"Godotゲームはまだ完全にはサポートされていません。",
		engineGameMakerNotFullySupported:
			"GameMakerゲームはまだ完全にはサポートされていません。",
	},

	providerCommand: {
		installGame: "インストール",
		showGameInLibrary: "ライブラリで表示",
		showGameInStore: "ストアページを開く",
		startGameViaProvider: "ゲームを開始",
		startGameViaExe: "ゲーム実行ファイルを実行",
		openGamePageInBrowser: "ブラウザで開く",
	},

	gameModal: {
		startGameButton: "ゲームを開始",
		startGameExecutable: "実行ファイルでゲームを開始",
		startGameViaProvider: "{provider}でゲームを開始",
		foldersDropdown: "フォルダー",
		openGameFilesFolder: "ゲームファイルフォルダーを開く",
		openInstalledModsFolder: "インストール済みモッドフォルダーを開く",
		removeFromRaiPal: "Rai Palから削除",
		removeGameConfirmation: "このゲームをRai Palから削除してもよろしいですか？",
		refreshGame: "更新",
		failedToReadGameInfo:
			"このゲームに関する重要な情報の一部を読み取ることができませんでした。実行ファイルが保護されている可能性があります。一部のモッドがインストールに失敗する可能性があります。",
		failedToDetermineEngine:
			"このゲームのエンジンを特定できませんでした。一部のモッドがインストールに失敗する可能性があります。",
		gameModsLabel: "モッド",
		gameNotInstalledWarning:
			"このゲームはインストールされていないため、どのモッドが互換性があるか100％確信できません。以下に表示されるモッドは動作する可能性があります。ゲームをインストールすると、より正確な情報を表示します。",
		uninstallAllModsButton: "すべてのモッドをアンインストール",
		uninstallAllModsConfirmation:
			"本当に？これにより、このゲームのモッドフォルダー内のすべてのファイルが削除されます。ただし、実際のゲームのファイルは削除されません。",

		incompatibleGameModsLabel: "非互換モッド",

		incompatibleGameModsDescription:
			"ここにリストされているモッドは、このゲームのエンジンバージョンと互換性がないため、インストールできません。",
	},

	gameModRow: {
		editModConfig: "モッド設定を編集",
		openModConfigFolderTooltip:
			"このモッドの設定ファイルが含まれているフォルダーを開く",
		openModFolder: "モッドフォルダーを開く",
		updateMod: "更新",
		installMod: "インストール",
		uninstallMod: "アンインストール",
		runMod: "実行",
		downloadRemoteConfig: "推奨設定をダウンロード",
		remoteConfigAvailable:
			"推奨設定が利用可能です。設定がまだない場合にダウンロードされます。三点メニューから強制的にダウンロードすることもできます。",
	},

	gamesTableColumn: {
		game: "ゲーム",
		engine: "エンジン",
		date: "日付",
	},

	modsPage: {
		openModsFolderButton: "モッドフォルダーを開く",
		tableColumnMod: "モッド",
		tableColumnVersion: "バージョン",
		tableColumnModLoader: "ローダー",
		tableColumnGameEngine: "エンジン",
		tableColumnUnityBackend: "バックエンド",
		modByAuthor: "{authorName} 作",
		modDeprecated: "非推奨",
		modDeprecatedTooltip:
			"このモッドは非推奨です。アンインストールして新しい代替品をインストールすることをお勧めします。",
		modOutdated: "モッドが古い",
	},

	modModal: {
		runMod: "実行",
		openModFolder: "モッドフォルダーを開く",
		updateMod: "モッドを更新",
		downloadMod: "モッドをダウンロード",
		deleteMod: "モッドを削除",
		deleteModConfirmation:
			"本当に？モッドフォルダー内のファイルはすべて失われます。",
		byAuthor: "{authorName} 作",
	},

	appDropdownMenu: {
		showGameThumbnails: "リストにゲームのサムネイルを表示",
		language: "言語",
		autoDetectedLanguage: "自動検出 - {languageName}",
		resetRaiPalSettingsButton: "Rai Palの設定をリセット",
		resetRaiPalSettingsTooltip:
			"フィルター、確認ダイアログ、おそらく他の設定をリセットします。",
		openLogsFolderButton: "ログフォルダーを開く",
		clearRaiPalCacheOpenModal: "Rai Palのキャッシュをクリア",
		clearRaiPalCacheTooltip:
			"Rai Palが使用するゲームリストキャッシュをクリアします。",
	},

	steamCache: {
		resetSteamCacheButton: "Steamのキャッシュをリセット",
		resetSteamCacheModalTitle: "Steamのキャッシュをリセット",
		resetSteamCacheDescription:
			"Rai Palが実際に所有していないSteamのゲームを表示している場合に使用します。これによりSteamのキャッシュがリセットされ、Steamを再起動する必要があります。ファイルが既に削除されている場合はエラーが発生します。",
		resetSteamCacheSuccess:
			"キャッシュファイルが削除されました。Steamを再起動し、数秒待ってからRai Palの更新ボタンを押してください。",
	},

	debugData: {
		debugDataTitle: "デバッグデータ",
		debugDataCopy: "デバッグデータをコピー",
	},

	thanksPage: {
		intro:
			"こんにちは。私はRai Palを作りました。過去には他のVRモッドも作成しており、現在はUnityゲーム用のユニバーサルVRモッドに取り組んでいます。私の活動を気に入っていただけたら、寄付を検討してください。また、itch.ioで私の無料モッドの1つを購入してサポートすることもできます。",
		starRaiPalOnGitHub: "GitHubでRai Palにスターを付ける",
		otherModdersTitle: "他のモッダー",
		otherModdersDescription:
			"Rai Palはゲームのモッディングを支援することを目的としており、他の開発者が作成したツールなしではそれを実現できません。これらの人々の中には寄付リンクを持っていない人もいますが、彼らの仕事に非常に感謝しています。",
		modderOnWebsite: "{modderName} の {website}",
		patreonLeaderboard: "Patreonリーダーボード",
		rankedByPatreonDonationAmount: "総生涯寄付額によるランキング。",
		patreonProfilePrivateNotice:
			"ここに表示されない場合は、Patreonプロファイルが非公開であるためです。",
	},

	commandButton: {
		cancel: "キャンセル",
		dontAskAgain: "今後確認しない",
	},
} as const;
