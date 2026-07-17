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
		mod: "Mod",
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
		openGameFilesFolder: "ゲームの場所",
		openInstalledModsFolder: "導入済みMod",
		openGameDataFolder: "アプリデータ",
		openGameWinePrefixFolder: "Wineプレフィックス",
		openGameWineBinaryFolder: "Wineバイナリ",
		removeFromRaiPal: "Rai Palから削除",
		removeGameConfirmation: "このゲームをRai Palから削除していい？",
		refreshGame: "更新",
		failedToReadGameInfo:
			"このゲームの重要情報がちゃんと読み取れなかった。実行ファイルが保護されてるのかも。一部のModはうまくインストールできないかも。",
		failedToDetermineEngine:
			"このゲームのエンジンが特定できなかった。一部のModはうまくインストールできないかも。",
		gameModsLabel: "Mod",
		gameNotInstalledWarning:
			"このゲームはインストールされてないから、どのModが対応してるか100%は確信できないんだ。下に出てるModなら動くかも。インストールしたら、もっと正確な情報を出せるよ。",
		uninstallAllModsButton: "すべてのModをアンインストール",
		uninstallAllModsConfirmation:
			"本当に？このゲームのModフォルダー内の全ファイルが消えるよ。でもゲーム本体のファイルは消さないからね。",

		incompatibleGameModsLabel: "非対応Mod",

		incompatibleGameModsDescription:
			"ここに表示されてるModは、このゲームのエンジンバージョンと互換性がないからインストールできないよ。",

		otherThings: "その他のもの",

		otherThingsDescription:
			"これらは主に依存関係とかで、普段は直接いじる必要はないよ。",
	},

	gameModRow: {
		editModConfig: "Mod設定",
		openModConfigFolderTooltip:
			"このModの設定ファイルがあるフォルダーを開く",
		openModFolder: "Modフォルダーを開く",
		updateMod: "更新",
		installMod: "インストール",
		installModAnticheatWarning:
			"注意: マルチプレイゲームにModを入れるときは気をつけて！アンチチートがModを検出して、無害そうでもBANされることがあるからね。",
		reinstallMod: "再インストール",
		uninstallMod: "アンインストール",
		runMod: "実行",
		downloadRemoteConfig: "推奨設定をダウンロード",
		remoteConfigAvailable:
			"推奨設定あり。まだ設定がなければ自動でダウンロードされるよ。三点メニューから手動でもダウンロードできる。",
		modOutdated: "Modが古い",
		cantUninstallModWithDependants:
			"依存ModがあるModはアンインストールできないよ。先に依存してるModをアンインストールしてね。",
	},

	gamesTableColumn: {
		game: "ゲーム",
		engine: "エンジン",
		date: "日付",
	},

	modsPage: {
		openLocalModsFolderButton: "Modフォルダーを開く",
		openLoadlModsFolderTooltip:
			"このフォルダーに rai-pal-manifest.json 付きのModを入れれば、オンラインデータベースを通さずに直接読み込めるよ。",
		tableColumnMod: "Mod",
		tableColumnVersion: "バージョン",
		tableColumnGameEngine: "エンジン",
		tableColumnUnityBackend: "バックエンド",
		modByAuthor: "{authorName} 作",
		modDeprecated: "非推奨",
		modDeprecatedTooltip:
			"このModは非推奨だよ。アンインストールして新しいのに切り替えよう。",
	},

	modModal: {
		runMod: "実行",
		openModFolder: "Modフォルダーを開く",
		updateMod: "Modを更新",
		downloadMod: "Modをダウンロード",
		deleteMod: "Modを削除",
		deleteModConfirmation:
			"本当に？Modフォルダーの中のファイルが全部消えちゃうよ。",
		byAuthor: "{authorName} 作",
	},

	appDropdownMenu: {
		showGameThumbnails: "リストにゲームのサムネイルを表示",
		language: "言語",
		autoDetectedLanguage: "自動検出 - {languageName}",
		resetRaiPalSettingsButton: "Rai Palの設定をリセット",
		resetRaiPalSettingsTooltip:
			"フィルターや確認ダイアログ、あと多分その他の設定もリセットされるよ。",
		openLogsFolderButton: "ログフォルダーを開く",
		clearRaiPalCacheOpenModal: "Rai Palのキャッシュをクリア",
		clearRaiPalCacheTooltip:
			"Rai Palが使ってるゲームリストのキャッシュを消すよ。",
	},

	userMenu: {
		unknownUser: "不明なユーザー",
		logOut: "ログアウト",
		signInWithDiscord: "Discordでサインイン",
		discordAccessNote:
			"Mod はこれを使ってあなたのDiscordユーザー名、アバター、ロールなどにアクセスできます。",
	},

	subPage: {
		back: "戻る",
	},

	downloadStatusMenu: {
		clear: "完了したものを消す",
	},

	steamCache: {
		resetSteamCacheButton: "Steamのキャッシュをリセット",
		resetSteamCacheModalTitle: "Steamのキャッシュをリセット",
		resetSteamCacheDescription:
			"Rai Palが実際には持ってないSteamのゲームを表示してるときに使ってね。Steamのキャッシュがリセットされるから、その後Steamを再起動して。ファイルが既に消えてたらエラーが出るかも。",
		resetSteamCacheSuccess:
			"キャッシュファイルを消したよ。Steamを再起動して数秒待ったら、Rai Palの更新ボタンを押してね。",
	},

	steamShortcut: {
		addRaiPalSteamShortcutButton: "Rai PalをSteamライブラリに追加",
		addRaiPalSteamShortcutModalTitle: "Rai PalをSteamライブラリに追加",
		addRaiPalSteamShortcutDescription:
			"特にSteam DeckのゲームモードからRai Palを起動できるようになるから便利だよ。実行したらSteamを再起動してね。",
		addRaiPalSteamShortcutSuccess:
			"Rai PalをSteamライブラリに追加したよ。表示するにはSteamを再起動してね。",
	},

	globalWineOverrides: {
		setUpEnvironmentButton: "BepInEx用のLinux環境をセットアップ",
		setUpEnvironmentTitle: "BepInEx用のLinux環境をセットアップ",
		setUpEnvironmentDescription:
			"LinuxでProton/Wineを使ってると、Wineの設定をしないとBepInExは自動で読み込まれないんだ。これは環境変数 WINEDLLOVERRIDES をグローバルに 'winhttp.dll=n,b' に設定するよ。手動でやりたいなら、このボタンは押さないでね。",
		setUpEnvironmentSuccess:
			"ファイルを書き込んだよ。変更を反映するには、ログアウトして再ログインするか、PCを再起動してね。",
	},

	debugData: {
		debugDataTitle: "デバッグデータ",
		debugDataCopy: "デバッグデータをコピー",
	},

	thanksPage: {
		intro:
			"こんにちは。私がRai Palを作ったよ。過去には他のVR Modも作ってて、今はUnityゲーム向けのユニバーサルVR Modを開発中。もし私のやってることが気に入ったら、寄付を考えてみてね！itch.ioで私の無料Modを「購入」する形でもサポートできるよ。",
		starRaiPalOnGitHub: "GitHubでRai Palにスターを付ける",
		otherModdersTitle: "他のModder",
		otherModdersDescription:
			"Rai PalはゲームのMod管理を手助けするツールだけど、他の開発者が作ったツールなしじゃ成り立たないんだ。寄付リンクを持ってない人もいるけど、みんなの仕事には本当に感謝してるよ。",
		modderOnWebsite: "{modderName}（{website}）",
		patreonLeaderboard: "Patreonリーダーボード",
		rankedByPatreonDonationAmount: "累計寄付額でランク付け",
		patreonProfilePrivateNotice:
			"ここに表示されてない人は、Patreonのプロフィールが非公開になってるからだよ。",
	},

	commandButton: {
		cancel: "キャンセル",
		dontAskAgain: "今後確認しない",
	},

	urlModSources: {
		title: "Modソース",
		add: "ソースを追加",
		addSourceDescription:
			"Rai Palはデフォルトのデータベースに加えて、ここで提供されているすべてのデータベースでModをチェックします。ディープリンク {deepLink} でModソースを共有できます。",
		confirmModalTitle: "Modソースを確認",
		modsFound: "このソースに{count}個のModが見つかりました",
		addSource: "ソースを追加",
		cancel: "キャンセル",
		loading: "Modを読み込み中...",
	},
} as const;
