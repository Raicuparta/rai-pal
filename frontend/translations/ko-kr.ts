import { Translation } from "./translations";

export const koKr: Translation = {
	tab: {
		games: "게임",
		mods: "모드",
		tools: "도구",
		thanks: "감사",
	},

	addGame: {
		button: "게임 추가",
		title: "게임 추가",
		dropField:
			"여기에 게임 실행 파일을 드래그 앤 드롭하거나 클릭하여 파일을 선택하세요.",
		note: "참고: Rai Pal 창 어디에서든 게임 실행 파일을 드래그 앤 드롭하면 이 대화 상자를 열지 않고도 게임 목록에 추가할 수 있습니다.",
	},

	refresh: {
		button: "새로고침",
		loading: "{items} 불러오는 중...",
	},

	filterMenu: {
		button: "필터",
		resetButton: "초기화",
		searchPlaceholder: "검색...",
	},

	filterProperty: {
		provider: "제공자",
		tags: "태그",
		architecture: "아키텍처",
		unityScriptingBackend: "Unity 백엔드",
		engine: "엔진",
		status: "상태",
	},

	filterValue: {
		unknown: "알 수 없음",
		arch64: "64비트",
		arch32: "32비트",
		tagDemo: "데모",
		tagVr: "네이티브 VR",
		tagUntagged: "태그 없음",
		statusInstalled: "설치됨",
		statusNotInstalled: "미설치",
		providerManual: "수동",
	},

	filterValueNote: {
		providerXboxOnlyInstalled:
			"Xbox PC 게임은 설치된 경우에만 Rai Pal에서 표시됩니다.",
		engineGodotNotFullySupported: "Godot 게임은 아직 완전히 지원되지 않습니다.",
		engineGameMakerNotFullySupported:
			"GameMaker 게임은 아직 완전히 지원되지 않습니다.",
	},

	providerCommand: {
		installGame: "설치",
		showGameInLibrary: "라이브러리에서 보기",
		showGameInStore: "스토어 페이지 열기",
		startGame: "게임 시작",
		openGamePageInBrowser: "브라우저에서 열기",
	},

	gameModal: {
		startGameButton: "게임 시작",
		startGameExecutable: "실행 파일로 게임 시작",
		startGameViaProvider: "{provider} 통해 게임 시작",
		foldersDropdown: "폴더",
		openGameFilesFolder: "게임 파일 폴더 열기",
		openInstalledModsFolder: "설치된 모드 폴더 열기",
		removeFromRaiPal: "Rai Pal에서 제거",
		removeGameConfirmation: "이 게임을 Rai Pal에서 제거하시겠습니까?",
		refreshGame: "새로고침",
		failedToReadGameInfo:
			"이 게임의 중요한 정보를 읽어오는 데 실패했습니다. 실행 파일이 보호되어 있을 수 있습니다. 일부 모드 설치가 실패할 수 있습니다.",
		failedToDetermineEngine:
			"이 게임의 엔진을 확인할 수 없습니다. 일부 모드 설치가 실패할 수 있습니다.",
		gameModsLabel: "모드",
		gameNotInstalledWarning:
			"이 게임이 설치되지 않았으므로, 어떤 모드가 호환되는지 확실하지 않습니다. 아래에 표시된 모드가 작동할 수 있습니다. 게임을 설치하면 더 정확한 정보를 제공할 수 있습니다.",
		uninstallAllModsButton: "모든 모드 제거",
		uninstallAllModsConfirmation:
			"정말로 제거하시겠습니까? 이 게임의 모드 폴더에 있는 모든 파일이 삭제됩니다. 하지만 게임 자체 파일은 삭제되지 않습니다.",
	},

	gamesTableColumn: {
		game: "게임",
		engine: "엔진",
		date: "날짜",
	},

	modsPage: {
		openModsFolderButton: "모드 폴더 열기",
		tableColumnMod: "모드",
		tableColumnVersion: "버전",
		tableColumnModLoader: "로더",
		tableColumnGameEngine: "엔진",
		tableColumnUnityBackend: "백엔드",
		modByAuthor: "{authorName} 제작",
		modDeprecated: "더 이상 지원되지 않음",
		modDeprecatedTooltip:
			"이 모드는 더 이상 지원되지 않습니다. 새로운 대체 모드를 설치하는 것이 좋습니다.",
		modOutdated: "모드 구버전",
	},

	modModal: {
		runMod: "실행",
		openModFolder: "모드 폴더 열기",
		updateMod: "모드 업데이트",
		downloadMod: "모드 다운로드",
		deleteMod: "모드 삭제",
		deleteModConfirmation:
			"정말로 삭제하시겠습니까? 이 모드 폴더의 모든 파일이 삭제됩니다.",
		byAuthor: "{authorName} 제작",
	},

	toolsPage: {
		openLogsFolderButton: "로그 폴더 열기",
		resetRaiPalSettingsButton: "Rai Pal 설정 초기화",
		resetRaiPalSettingsTooltip:
			"필터, 확인 대화 상자 및 기타 설정을 초기화합니다.",
		clearCacheButton: "Rai Pal 캐시 삭제",
		clearCacheTooltip: "Rai Pal이 사용하는 게임 목록 캐시를 삭제합니다.",
	},

	steamCache: {
		resetSteamCacheButton: "Steam 캐시 초기화",
		resetSteamCacheModalTitle: "Steam 캐시 초기화",
		resetSteamCacheDescription:
			"Steam에 실제로 소유하지 않은 게임이 표시될 경우 사용하세요. Steam 캐시를 초기화하고 Steam을 다시 시작해야 합니다.",
		resetSteamCacheSuccess:
			"캐시 파일이 삭제되었습니다. Steam을 다시 시작한 후 Rai Pal에서 새로고침 버튼을 눌러 주세요.",
	},

	debugData: {
		debugDataTitle: "디버그 데이터",
		debugDataCopy: "디버그 데이터 복사",
	},

	thanksPage: {
		intro:
			"안녕하세요. Rai Pal을 만들었습니다. 다양한 VR 모드를 개발했으며, Unity 게임용 범용 VR 모드를 작업 중입니다. 지원을 원하시면 기부를 고려해 주세요!",
		starRaiPalOnGitHub: "GitHub에서 Rai Pal 즐겨찾기",
		otherModdersTitle: "다른 모더들",
		otherModdersDescription:
			"Rai Pal은 게임 모딩을 돕기 위해 만들어졌으며, 다른 개발자들의 도구 없이는 불가능합니다.",
		modderOnWebsite: "{modderName} ({website})",
		patreonLeaderboard: "Patreon 리더보드",
		rankedByPatreonDonationAmount: "기부 총액 순위",
		patreonProfilePrivateNotice:
			"프로필이 비공개 설정되어 있으면 여기에 표시되지 않습니다.",
	},

	commandButton: {
		cancel: "취소",
		dontAskAgain: "다시 묻지 않기",
	},
};
