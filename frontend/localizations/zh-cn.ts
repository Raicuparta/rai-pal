import { Localization } from "./localizations";

export const zhCn: Localization = {
	meta: {
		nativeName: "简体中文",
	},

	tab: {
		games: "游戏",
		mods: "模组",
		thanks: "致谢",
	},

	gamesPage: {
		emptyGamesList:
			"Rai Pal 一个游戏都没找到。别忘了 Rai Pal 需要从其他游戏平台（如 Steam、Epic 等）检测到已安装的应用。",
		emptyFilteredGamesList:
			"什么都没有！因为你选的筛选条件，所有游戏都被隐藏了。清除筛选，再看看你的游戏吧。",
		emptyGamesLoading: "正在找你的游戏...",
	},

	manualGames: {
		manualSteamSupportNote:
			"注意：你也可以将非 Steam 游戏添加到 Steam 中，Rai Pal 仍会检测到它们。",
		savedDirectories:
			"每次 Rai Pal 刷新数据时，这些目录都将被递归扫描。大文件夹可能会显著降低速度。",
		button: "添加游戏",
		title: "手动添加的游戏",
		selectGameExecutable: "选择游戏可执行文件",
		selectGamesDirectory: "选择游戏目录。可能很慢！",
		fileDropNote:
			"你也可以将游戏可执行文件或整个文件夹拖放到 Rai Pal 窗口的任何位置来添加到库中，无需打开此对话框。",
		scanning: "正在扫描 {path}...",
		scanProgress:
			"已扫描 {directories} 个目录，找到 {executables} 个可执行文件",
		scanComplete:
			"在 {duration} 秒内找到 {gamesCount} 个可执行文件。是否将此文件夹添加到 Rai Pal？",
		confirmAddFolder: "添加文件夹",
		cancel: "取消",
	},

	refresh: {
		button: "刷新",
		buttonUpdateRemoteDatabases: "更新远程数据库",
		loading: "正在加载 {items}...",
	},

	filterMenu: {
		button: "筛选",
		resetButton: "重置",
		searchPlaceholder: "搜索...",
	},

	filterProperty: {
		provider: "平台",
		tags: "标签",
		architecture: "架构",
		unityBackend: "Unity 后端",
		engine: "引擎",
		status: "状态",
		mod: "模组",
	},

	filterValue: {
		unknown: "未知",
		arch64: "64 位",
		arch32: "32 位",
		tagDemo: "试玩",
		tagVr: "原生 VR",
		tagUntagged: "未标记",
		statusInstalled: "已安装",
		statusNotInstalled: "未安装",
		providerManual: "手动",
	},

	filterValueNote: {
		providerXboxOnlyInstalled: "仅在已安装时显示 PC Xbox 游戏。",
		engineGodotNotFullySupported: "Godot 游戏尚未完全支持。",
		engineGameMakerNotFullySupported: "GameMaker 游戏尚未完全支持。",
	},

	providerCommand: {
		installGame: "安装",
		showGameInLibrary: "在库中显示",
		showGameInStore: "打开商店页面",
		startGameViaProvider: "启动游戏",
		startGameViaExe: "运行游戏可执行文件",
		openGamePageInBrowser: "在浏览器中打开",
	},

	gameModal: {
		startGameButton: "启动游戏",
		startGameExecutable: "启动游戏可执行文件",
		startGameViaProvider: "通过 {provider} 启动游戏",
		foldersDropdown: "文件夹",
		openGameFilesFolder: "打开游戏文件夹",
		openInstalledModsFolder: "打开已安装模组文件夹",
		openGameDataFolder: "打开游戏数据文件夹",
		openGameWinePrefixFolder: "打开游戏的 Wine 前缀文件夹",
		openGameWineBinaryFolder: "打开 Wine 程序文件夹",
		removeFromRaiPal: "从 Rai Pal 移除",
		removeGameConfirmation: "确定要从 Rai Pal 移除此游戏吗？",
		refreshGame: "刷新",
		failedToReadGameInfo:
			"无法读取此游戏的部分重要信息。可执行文件可能被保护了。一些模组可能无法安装。",
		failedToDetermineEngine: "无法确定此游戏的引擎。一些模组可能无法安装。",
		gameModsLabel: "模组",
		gameNotInstalledWarning:
			"此游戏未安装，所以我不太确定哪些模组兼容。下面列出的模组可能可以用。安装游戏后，我再显示更准确的信息。",
		uninstallAllModsButton: "卸载所有模组",
		uninstallAllModsConfirmation:
			"你确定吗？这将删除此游戏模组文件夹中的所有文件。但不会删除游戏本身的文件。",

		incompatibleGameModsLabel: "不兼容的模组",

		incompatibleGameModsDescription:
			"这里列出的模组因不兼容此游戏的引擎版本，无法安装。",

		otherThings: "其他东西",

		otherThingsDescription: "这些主要是依赖项之类的，一般不用直接碰。",
	},

	gameModRow: {
		editModConfig: "编辑模组配置",
		openModConfigFolderTooltip: "打开包含此模组配置文件的文件夹",
		openModFolder: "打开模组文件夹",
		updateMod: "更新",
		installMod: "安装",
		installModAnticheatWarning:
			"警告：在多人游戏中安装模组时要小心！反作弊系统可能会检测到某些模组并导致封号，即使这些模组看起来无害。",
		reinstallMod: "重新安装",
		uninstallMod: "卸载",
		runMod: "运行",
		downloadRemoteConfig: "下载推荐配置",
		remoteConfigAvailable:
			"推荐配置可用。如果你还没有配置，将会下载。你也可以从三点菜单强制下载。",
		modOutdated: "模组过时",
		cantUninstallModWithDependants:
			"无法卸载有依赖项的模组。请先卸载依赖此模组的其他模组。",
	},

	gamesTableColumn: {
		game: "游戏",
		engine: "引擎",
		date: "日期",
	},

	modsPage: {
		openLocalModsFolderButton: "打开本地模组文件夹",
		openLoadlModsFolderTooltip:
			"在此文件夹中放入包含 rai-pal-manifest.json 的模组，即可直接加载，无需通过在线数据库。",
		tableColumnMod: "模组",
		tableColumnVersion: "版本",
		tableColumnGameEngine: "引擎",
		tableColumnUnityBackend: "后端",
		modByAuthor: "由 {authorName} 制作",
		modDeprecated: "已弃用",
		modDeprecatedTooltip: "此模组已弃用。您应该卸载它并安装更新的替代品。",
	},

	modModal: {
		runMod: "运行",
		openModFolder: "打开模组文件夹",
		updateMod: "更新模组",
		downloadMod: "下载模组",
		deleteMod: "删除模组",
		deleteModConfirmation: "你确定吗？模组文件夹中的任何文件都将丢失。",
		byAuthor: "由 {authorName} 制作",
	},

	appDropdownMenu: {
		showGameThumbnails: "在列表中显示游戏缩略图",
		language: "语言",
		autoDetectedLanguage: "自动检测 - {languageName}",
		resetRaiPalSettingsButton: "重置 Rai Pal 设置",
		resetRaiPalSettingsTooltip:
			"将重置筛选条件、确认对话框，可能还有其他内容。",
		openLogsFolderButton: "打开日志文件夹",
		clearRaiPalCacheOpenModal: "清除 Rai Pal 缓存",
		clearRaiPalCacheTooltip: "清除 Rai Pal 使用的游戏列表缓存。",
	},

	userMenu: {
		unknownUser: "未知用户",
		logOut: "退出登录",
		signInWithDiscord: "使用 Discord 登录",
		discordAccessNote:
			"模组可以使用此功能访问你的 Discord 用户名、头像、身份组等。",
	},

	subPage: {
		back: "返回",
	},

	downloadStatusMenu: {
		clear: "清除已完成",
	},

	steamCache: {
		resetSteamCacheButton: "重置 Steam 缓存",
		resetSteamCacheModalTitle: "重置 Steam 缓存",
		resetSteamCacheDescription:
			"如果 Rai Pal 显示了你不拥有的 Steam 游戏，可以用此功能。这会重置 Steam 的缓存，然后你需要重启 Steam。如果文件已被删除，会显示错误。",
		resetSteamCacheSuccess:
			"缓存文件已被删除。请重启 Steam，等待几秒钟，然后按 Rai Pal 上的刷新按钮。",
	},

	steamShortcut: {
		addRaiPalSteamShortcutButton: "将 Rai Pal 添加到 Steam 库",
		addRaiPalSteamShortcutModalTitle: "将 Rai Pal 添加到 Steam 库",
		addRaiPalSteamShortcutDescription:
			"这在 Steam Deck 上尤其有用，这样你就可以在游戏模式中启动 Rai Pal。完成后需要重启 Steam。",
		addRaiPalSteamShortcutSuccess:
			"Rai Pal 已添加到你的 Steam 库中。重启 Steam 后即可看到它。",
	},

	globalWineOverrides: {
		setUpEnvironmentButton: "为 BepInEx 设置 Linux 环境",
		setUpEnvironmentTitle: "为 BepInEx 设置 Linux 环境",
		setUpEnvironmentDescription:
			"在 Linux 上使用 Proton/Wine 时，除非设置了一些 Wine 配置，否则 BepInEx 不会自动加载。此操作将全局设置环境变量 WINEDLLOVERRIDES 为 'winhttp.dll=n,b'。如果你更想手动设置，别点这个按钮。",
		setUpEnvironmentSuccess:
			"文件已写入。你需要注销后重新登录，或重启计算机，更改才能生效。",
	},

	debugData: {
		debugDataTitle: "调试数据",
		debugDataCopy: "复制调试数据",
	},

	thanksPage: {
		intro:
			"你好。我做了 Rai Pal。我也做过其他 VR 模组，目前正在开发一个通用的 Unity 游戏 VR 模组。如果你喜欢我的作品并想看更多，可以考虑捐赠！你也可以在 itch.io 上购买我的免费模组来支持我。",
		starRaiPalOnGitHub: "在 GitHub 上为 Rai Pal 点星",
		otherModdersTitle: "其他模组作者",
		otherModdersDescription:
			"Rai Pal 的目的是帮你管理游戏模组，而我们离不开其他开发者创建的工具。这些人中有些没有捐赠链接，但我非常感谢他们的工作。",
		modderOnWebsite: "{modderName} 在 {website}",
		patreonLeaderboard: "Patreon 排行榜",
		rankedByPatreonDonationAmount: "按总捐赠金额排名。",
		patreonProfilePrivateNotice:
			"如果你没看到自己，那是因为你的 Patreon 个人资料是私密的。",
	},

	commandButton: {
		cancel: "取消",
		dontAskAgain: "不再询问",
	},

	urlModSources: {
		title: "模组来源",
		add: "添加来源",
		addSourceDescription:
			"Rai Pal 除了默认数据库外，还会在此提供的所有数据库中检查模组。你可以通过深层链接 {deepLink} 共享模组来源。",
		confirmModalTitle: "确认模组来源",
		modsFound: "在此来源中找到 {count} 个模组",
		addSource: "添加来源",
		cancel: "取消",
		loading: "正在加载模组...",
	},
} as const;
