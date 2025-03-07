import { Translation } from "./translations";

export const zhCn: Translation = {
	meta: {
		nativeName: "简体中文",
	},
	tab: {
		games: "游戏",
		mods: "模组",
		tools: "工具",
		thanks: "鸣谢",
	},
	addGame: {
		button: "添加游戏",
		title: "添加游戏",
		dropField: "将游戏可执行文件拖放到此处，或点击选择文件。",
		note: "注意：您可以将游戏可执行文件拖放到 Rai Pal 窗口的任何位置，以将其添加到已安装的游戏列表，而无需打开此对话框。",
	},
	refresh: {
		button: "刷新",
		loading: "加载 {items}...",
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
		unityScriptingBackend: "Unity 后端",
		engine: "引擎",
		status: "状态",
	},
	filterValue: {
		unknown: "未知",
		arch64: "64 位",
		arch32: "32 位",
		tagDemo: "演示版",
		tagVr: "原生 VR",
		tagUntagged: "未标记",
		statusInstalled: "已安装",
		statusNotInstalled: "未安装",
		providerManual: "手动",
	},
	filterValueNote: {
		providerXboxOnlyInstalled:
			"Xbox PC 游戏只有在安装后才会出现在 Rai Pal 中。",
		engineGodotNotFullySupported: "Godot 游戏尚未完全支持。",
		engineGameMakerNotFullySupported: "GameMaker 游戏尚未完全支持。",
	},
	providerCommand: {
		installGame: "安装",
		showGameInLibrary: "在库中显示",
		showGameInStore: "打开商店页面",
		startGame: "启动游戏",
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
			"无法读取此游戏的一些重要信息。这可能是由于可执行文件受保护。某些模组可能无法安装。",
		failedToDetermineEngine: "无法确定此游戏的引擎。某些模组可能无法安装。",
		gameModsLabel: "模组",
		gameNotInstalledWarning:
			"此游戏未安装，因此无法确保哪些模组兼容。以下模组可能可用。如果您安装游戏，我将显示更准确的信息。",
		uninstallAllModsButton: "卸载所有模组",
		uninstallAllModsConfirmation:
			"确定吗？这将删除该游戏模组文件夹中的所有文件，但不会删除游戏本身的文件。",
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
		modByAuthor: "作者：{authorName}",
		modDeprecated: "已弃用",
		modDeprecatedTooltip: "此模组已弃用。您应该卸载它并安装更新的替代品。",
		modOutdated: "模组已过时",
	},
	modModal: {
		runMod: "运行",
		openModFolder: "打开模组文件夹",
		updateMod: "更新模组",
		downloadMod: "下载模组",
		deleteMod: "删除模组",
		deleteModConfirmation: "确定吗？模组文件夹中的所有文件都将丢失。",
		byAuthor: "作者：{authorName}",
	},
	toolsPage: {
		openLogsFolderButton: "打开日志文件夹",
		resetRaiPalSettingsButton: "重置 Rai Pal 设置",
		resetRaiPalSettingsTooltip: "将重置筛选器、确认对话框等。",
		clearCacheButton: "清除 Rai Pal 缓存",
		clearCacheTooltip: "清除 Rai Pal 使用的游戏列表缓存。",
	},
	appSettings: {
		showGameThumbnails: "显示游戏缩略图",
		language: "语言",
		autoDetectedLanguage: "自动检测 - {languageName}",
	},
	steamCache: {
		resetSteamCacheButton: "重置 Steam 缓存",
		resetSteamCacheModalTitle: "重置 Steam 缓存",
		resetSteamCacheDescription:
			"如果 Rai Pal 显示了您实际未拥有的 Steam 游戏，请使用此选项。这将重置 Steam 缓存，然后您需要重新启动 Steam。如果文件已被删除，则会出现错误。",
		resetSteamCacheSuccess:
			"缓存文件已删除。请重新启动 Steam，等待几秒钟，然后在 Rai Pal 中点击刷新按钮。",
	},
	debugData: {
		debugDataTitle: "调试数据",
		debugDataCopy: "复制调试数据",
	},
	thanksPage: {
		intro:
			"你好，我是 Rai Pal 的开发者。我曾开发过其他 VR 模组，并正在制作一个通用的 Unity 游戏 VR 模组。如果您喜欢我的工作并希望看到更多，请考虑捐赠！您还可以通过购买 itch.io 上的免费模组来支持我。",
		starRaiPalOnGitHub: "在 GitHub 上 Star Rai Pal",
		otherModdersTitle: "其他模组作者",
		otherModdersDescription:
			"Rai Pal 旨在帮助您管理游戏模组，而这离不开其他开发者的工具。",
		modderOnWebsite: "{modderName} 在 {website}",
		patreonLeaderboard: "Patreon 排行榜",
		rankedByPatreonDonationAmount: "按总捐赠金额排名。",
		patreonProfilePrivateNotice:
			"如果您没有出现在此列表中，可能是因为您的 Patreon 资料是私密的。",
	},
	commandButton: {
		cancel: "取消",
		dontAskAgain: "不再询问",
	},
};
