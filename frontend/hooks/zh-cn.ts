export const zhCn = {
	tab: {
		// 显示用户从所有提供商获取的游戏的选项卡名称。
		games: "游戏",

		// 显示所有可用模组的选项卡名称。
		mods: "模组",

		// 显示工具和设置的选项卡名称。
		tools: "工具",

		// 显示鸣谢和捐赠链接的选项卡名称。
		thanks: "感谢",
	},

	addGame: {
		// 向 Rai Pal 添加游戏的按钮。
		button: "添加游戏",

		// 用于添加游戏的对话框标题。
		title: "添加游戏",

		// 添加游戏时，文件拖放区域内的文本。
		dropField: "将游戏可执行文件拖拽到此处，或点击以选择文件。",

		// 在添加游戏的文件拖放区域下方的提示信息。
		note: "注意：你可以将游戏可执行文件拖拽到 Rai Pal 窗口的任意位置来将它们添加至已安装游戏列表，而无需打开此对话框。",
	},

	refresh: {
		// 刷新游戏和模组的按钮。
		button: "刷新",

		// 在刷新按钮中的小字，正在加载内容时显示。{items} 是一个逗号分隔的列表。
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
		unityScriptingBackend: "Unity 后端",
		engine: "引擎",
		status: "状态",
	},

	filterValue: {
		unknown: "未知",
		arch64: "64 位",
		arch32: "32 位",
		engineGodot: "Godot",
		engineGameMaker: "GameMaker",
		engineUnity: "Unity",
		engineUnreal: "Unreal",
		unityBackendIl2Cpp: "IL2CPP",
		unityBackendMono: "Mono",
		tagDemo: "演示版",
		tagVr: "原生 VR",
		tagUntagged: "未标记",
		statusInstalled: "已安装",
		statusNotInstalled: "未安装",
		providerSteam: "Steam",
		providerGog: "GOG",
		providerEpic: "Epic",
		providerItch: "itch.io",
		providerOrigin: "Origin",
		providerManual: "手动",
		providerXbox: "Xbox",
		providerEa: "EA",
		providerUbisoft: "Ubisoft",
	},

	filterValueNote: {
		providerXboxOnlyInstalled:
			"仅当 Xbox PC 游戏已安装时，Rai Pal 才会显示它们。",
		engineGodotNotFullySupported: "Godot 游戏尚未得到完全支持。",
		engineGameMakerNotFullySupported: "GameMaker 游戏尚未得到完全支持。",
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
		removeGameConfirmation: "确定要从 Rai Pal 中移除此游戏吗？",
		removeFromRaiPal: "从 Rai Pal 中移除",
		refreshGame: "刷新",
		failedToReadGameInfo:
			"读取此游戏的一些重要信息失败，这可能是由于可执行文件受到保护。一些模组可能无法安装。",
		failedToDetermineEngine: "无法确定此游戏的引擎。一些模组可能无法安装。",
		gameModsLabel: "模组",
		gameNotInstalledWarning:
			"此游戏尚未安装，因此我无法完全确认哪些模组兼容。你看到的模组可能可以使用。如果你安装了游戏，我将显示更准确的信息。",
		uninstallAllModsConfirmation:
			"确定吗？这将删除此游戏模组文件夹中的所有文件，但不会删除游戏的任何文件。",
		uninstallAllModsButton: "卸载所有模组",
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
		modDeprecatedTooltip: "此模组已弃用。建议卸载并改用更新的替代模组。",
		modOutdated: "模组版本已过期",
	},

	modModal: {
		runMod: "运行",
		openModFolder: "打开模组文件夹",
		updateMod: "更新模组",
		downloadMod: "下载模组",
		deleteMod: "删除模组",
		deleteModConfirmation: "确定吗？此操作将清空模组文件夹内的所有文件。",
		byAuthor: "作者：{authorName}",
	},

	toolsPage: {
		openLogsFolderButton: "打开日志文件夹",
		resetRaiPalSettingsButton: "重置 Rai Pal 设置",
		resetRaiPalSettingsTooltip:
			"这将重置筛选器、确认对话框以及可能的其他设置。",
		clearCacheButton: "清除 Rai Pal 缓存",
		clearCacheTooltip: "清除 Rai Pal 使用的游戏列表缓存。",
	},

	steamCache: {
		resetSteamCacheButton: "重置 Steam 缓存",
		resetSteamCacheModalTitle: "重置 Steam 缓存",
		resetSteamCacheDescription:
			"如果 Rai Pal 显示了你实际上并未拥有的 Steam 游戏，可使用此功能重置 Steam 缓存，然后你需要重启 Steam。如果文件已被删除，你会收到错误提示。",
		resetSteamCacheSuccess:
			"缓存文件已被删除。请重启 Steam，等待几秒钟，然后在 Rai Pal 上点击刷新按钮。",
	},

	debugData: {
		debugDataTitle: "调试数据",
		debugDataCopy: "复制调试数据",
	},

	thanksPage: {
		intro:
			"你好，我是 Rai Pal 的开发者。我还开发了一些 VR 模组，并正在为 Unity 游戏开发一个通用 VR 模组。如果你喜欢我的工作，并希望看到更多，请考虑捐赠！你也可以通过购买我在 itch.io 上的免费模组来支持我。",
		starRaiPalOnGitHub: "在 GitHub 上为 Rai Pal 点星",
		otherModdersTitle: "其他模组作者",
		otherModdersDescription:
			"Rai Pal 致力于帮助你管理游戏模组，没有其他开发者的工具就无法实现。一些作者没有捐赠链接，但我非常感谢他们的工作。",
		modderOnWebsite: "{modderName} 在 {website}",
		patreonLeaderboard: "Patreon 排行榜",
		rankedByPatreonDonationAmount: "根据 Patreon 历史捐赠金额进行排名。",
		patreonProfilePrivateNotice:
			"如果没有显示你的信息，可能是因为你的 Patreon 资料是私密的。",
	},

	commandButton: {
		cancel: "取消",
		dontAskAgain: "不再询问",
	},
} as const;
