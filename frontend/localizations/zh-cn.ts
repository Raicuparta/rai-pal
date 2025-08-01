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
			"Rai Pal 完全没有找到任何游戏。请记住 Rai Pal 需要从其他游戏提供商（如 Steam、Epic 等）找到已安装的应用程序。",
		emptyFilteredGamesList:
			"什么都没有！由于您选择的过滤器，您的所有游戏都被隐藏了。清除过滤器以再次查看您美丽的游戏。",
		emptyGamesLoading: "正在寻找您的游戏...",
	},

	addGame: {
		button: "添加游戏",
		title: "添加游戏",
		dropField: "拖放游戏可执行文件到这里，或点击选择文件。",
		note: "注意：您可以将游戏可执行文件拖放到 Rai Pal 窗口的任何地方，以将其添加到已安装游戏列表中，而无需打开此对话框。",
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
		provider: "提供商",
		tags: "标签",
		architecture: "架构",
		unityBackend: "Unity 后端",
		engine: "引擎",
		status: "状态",
	},

	filterValue: {
		unknown: "未知",
		arch64: "64 位",
		arch32: "32 位",
		tagDemo: "演示",
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
		removeFromRaiPal: "从 Rai Pal 移除",
		removeGameConfirmation: "您确定要从 Rai Pal 移除此游戏吗？",
		refreshGame: "刷新",
		failedToReadGameInfo:
			"无法读取有关此游戏的一些重要信息。这可能是由于可执行文件受保护。一些模组可能无法安装。",
		failedToDetermineEngine: "无法确定此游戏的引擎。一些模组可能无法安装。",
		gameModsLabel: "模组",
		gameNotInstalledWarning:
			"此游戏未安装，因此我不能 100% 确定哪些模组兼容。您看到的模组可能会工作。如果您安装了游戏，我会显示更准确的信息。",
		uninstallAllModsButton: "卸载所有模组",
		uninstallAllModsConfirmation:
			"您确定吗？这将删除此游戏模组文件夹中的所有文件。但不会删除游戏的任何文件。",

		incompatibleGameModsLabel: "不兼容的模组",

		incompatibleGameModsDescription:
			"此处列出的模组无法安装，因为它们与此游戏的引擎版本不兼容。",
	},

	gameModRow: {
		editModConfig: "编辑模组配置",
		openModConfigFolderTooltip: "打开包含此模组配置文件的文件夹",
		openModFolder: "打开模组文件夹",
		updateMod: "更新",
		installMod: "安装",
		uninstallMod: "卸载",
		runMod: "运行",
		downloadRemoteConfig: "下载推荐配置",
		remoteConfigAvailable:
			"推荐配置可用。如果您还没有配置，将会下载。您也可以从三点菜单强制下载。",
	},

	gamesTableColumn: {
		game: "游戏",
		engine: "引擎",
		date: "日期",
	},

	modsPage: {
		openModsFolderButton: "打开模组文件夹",
		tableColumnMod: "模组",
		tableColumnVersion: "版本",
		tableColumnModLoader: "加载器",
		tableColumnGameEngine: "引擎",
		tableColumnUnityBackend: "后端",
		modByAuthor: "作者 {authorName}",
		modDeprecated: "已弃用",
		modDeprecatedTooltip: "此模组已弃用。您应该卸载它并安装更新的替代品。",
		modOutdated: "模组过时",
	},

	modModal: {
		runMod: "运行",
		openModFolder: "打开模组文件夹",
		updateMod: "更新模组",
		downloadMod: "下载模组",
		deleteMod: "删除模组",
		deleteModConfirmation: "您确定吗？模组文件夹中的任何文件都将丢失。",
		byAuthor: "作者 {authorName}",
	},

	appDropdownMenu: {
		showGameThumbnails: "在列表中显示游戏缩略图",
		language: "语言",
		autoDetectedLanguage: "自动检测 - {languageName}",
		resetRaiPalSettingsButton: "重置 Rai Pal 设置",
		resetRaiPalSettingsTooltip: "将重置筛选器、确认对话框，可能还有其他内容。",
		openLogsFolderButton: "打开日志文件夹",
		clearRaiPalCacheOpenModal: "清除 Rai Pal 缓存",
		clearRaiPalCacheTooltip: "清除 Rai Pal 使用的游戏列表缓存。",
	},

	steamCache: {
		resetSteamCacheButton: "重置 Steam 缓存",
		resetSteamCacheModalTitle: "重置 Steam 缓存",
		resetSteamCacheDescription:
			"如果 Rai Pal 显示您实际上不拥有的 Steam 游戏，请使用此选项。这将重置 Steam 的缓存，然后您需要重新启动 Steam。如果文件已被删除，您将收到错误消息。",
		resetSteamCacheSuccess:
			"缓存文件已被删除。请重新启动 Steam，等待几秒钟，然后按 Rai Pal 上的刷新按钮。",
	},

	debugData: {
		debugDataTitle: "调试数据",
		debugDataCopy: "复制调试数据",
	},

	thanksPage: {
		intro:
			"你好。我制作了 Rai Pal。我还制作了其他 VR 模组，目前正在开发一个通用的 Unity 游戏 VR 模组。如果您喜欢我的作品，并希望看到更多，请考虑捐赠！您也可以通过在 itch.io 上购买我的免费模组来支持我。",
		starRaiPalOnGitHub: "在 GitHub 上为 Rai Pal 点星",
		otherModdersTitle: "其他模组作者",
		otherModdersDescription:
			"Rai Pal 旨在帮助您管理游戏模组，而我们离不开其他开发者创建的工具。这些人中有些没有捐赠链接，但我非常感谢他们的工作。",
		modderOnWebsite: "{modderName} 在 {website}",
		patreonLeaderboard: "Patreon 排行榜",
		rankedByPatreonDonationAmount: "按总捐赠金额排名。",
		patreonProfilePrivateNotice:
			"如果您没有看到自己，那是因为您的 Patreon 个人资料是私密的。",
	},

	commandButton: {
		cancel: "取消",
		dontAskAgain: "不再询问",
	},
} as const;
