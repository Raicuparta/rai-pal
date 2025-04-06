import { Localization } from "./localizations";

export const koKr: Localization = {
	meta: {
		nativeName: "한국어",
	},

	tab: {
		games: "게임",
		mods: "모드",
		thanks: "감사",
	},

	addGame: {
		button: "게임 추가",
		title: "게임 추가",
		dropField:
			"게임 실행 파일을 여기로 드래그 앤 드롭하거나, 클릭하여 파일을 선택하세요.",
		note: "참고: Rai Pal 창 어디에나 게임 실행 파일을 드롭하여 이 대화 상자를 열지 않고도 설치된 게임 목록에 추가할 수 있습니다.",
	},

	refresh: {
		button: "새로고침",
		loading: "{items} 로딩 중...",
	},

	filterMenu: {
		button: "필터",
		resetButton: "재설정",
		searchPlaceholder: "검색...",
	},

	filterProperty: {
		provider: "제공자",
		tags: "태그",
		architecture: "아키텍처",
		unityBackend: "Unity 백엔드",
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
		statusNotInstalled: "설치되지 않음",
		providerManual: "수동",
	},

	filterValueNote: {
		providerXboxOnlyInstalledAndSubscription:
			"PC Xbox 게임은 설치된 경우 또는 PC Game Pass 구독의 일부로 소유한 경우에만 표시됩니다.",
		providerUbisoftOnlySubscription:
			"Ubisoft+ 구독의 일부로 소유한 경우에만 Ubisoft 게임이 표시됩니다.",
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
		startGameViaProvider: "{provider}로 게임 시작",
		foldersDropdown: "폴더",
		openGameFilesFolder: "게임 파일 폴더 열기",
		openInstalledModsFolder: "설치된 모드 폴더 열기",
		removeFromRaiPal: "Rai Pal에서 제거",
		removeGameConfirmation: "이 게임을 Rai Pal에서 제거하시겠습니까?",
		refreshGame: "새로고침",
		failedToReadGameInfo:
			"이 게임에 대한 중요한 정보를 읽는 데 실패했습니다. 실행 파일이 보호되어 있을 수 있습니다. 일부 모드 설치가 실패할 수 있습니다.",
		failedToDetermineEngine:
			"이 게임의 엔진을 확인하는 데 실패했습니다. 일부 모드 설치가 실패할 수 있습니다.",
		gameModsLabel: "모드",
		gameNotInstalledWarning:
			"이 게임이 설치되지 않았기 때문에 어떤 모드가 호환되는지 100% 확신할 수 없습니다. 아래에 표시된 모드가 작동할 수 있습니다. 게임을 설치하면 더 정확한 정보를 보여드리겠습니다.",
		uninstallAllModsButton: "모드 모두 제거",
		uninstallAllModsConfirmation:
			"확실합니까? 이 게임의 모드 폴더에 있는 모든 파일이 삭제됩니다. 실제 게임의 파일은 삭제되지 않습니다.",
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
		modDeprecated: "사용 중지됨",
		modDeprecatedTooltip:
			"이 모드는 사용 중지되었습니다. 이 모드를 제거하고 새로운 대안을 설치해야 합니다.",
		modOutdated: "모드가 오래되었습니다",
	},

	modModal: {
		runMod: "실행",
		openModFolder: "모드 폴더 열기",
		updateMod: "모드 업데이트",
		downloadMod: "모드 다운로드",
		deleteMod: "모드 삭제",
		deleteModConfirmation: "확실합니까? 모드 폴더 안의 모든 파일이 삭제됩니다.",
		byAuthor: "{authorName} 제작",
	},

	appDropdownMenu: {
		showGameThumbnails: "목록에 게임 썸네일 표시",
		language: "언어",
		autoDetectedLanguage: "자동 감지 - {languageName}",
		resetRaiPalSettingsButton: "Rai Pal 설정 재설정",
		resetRaiPalSettingsTooltip:
			"필터, 확인 대화 상자 및 기타 설정을 재설정합니다.",
		openLogsFolderButton: "로그 폴더 열기",
		clearRaiPalCacheOpenModal: "Rai Pal 캐시 지우기",
		clearRaiPalCacheTooltip: "Rai Pal이 사용하는 게임 목록 캐시를 지웁니다.",
	},

	steamCache: {
		resetSteamCacheButtonOpenModal: "Steam 캐시 재설정...",
		resetSteamCacheModalTitle: "Steam 캐시 재설정",
		resetSteamCacheDescription:
			"Rai Pal이 실제로 소유하지 않은 Steam 게임을 표시하는 경우 이 옵션을 사용하세요. Steam의 캐시를 재설정한 후 Steam을 다시 시작해야 합니다. 파일이 이미 삭제된 경우 오류가 발생합니다.",
		resetSteamCacheSuccess:
			"캐시 파일이 삭제되었습니다. Steam을 다시 시작하고 몇 초 기다린 후 Rai Pal에서 새로고침 버튼을 누르세요.",
	},

	debugData: {
		debugDataTitle: "디버그 데이터",
		debugDataCopy: "디버그 데이터 복사",
	},

	thanksPage: {
		intro:
			"안녕하세요. 저는 Rai Pal을 만들었습니다. 과거에 다른 VR 모드를 만들었고, 현재는 Unity 게임을 위한 범용 VR 모드를 작업 중입니다. 제가 하는 일을 좋아하시고 더 많은 것을 보고 싶다면 기부를 고려해 주세요! itch.io에서 무료 모드 중 하나를 구매하여 저를 지원할 수도 있습니다.",
		starRaiPalOnGitHub: "GitHub에서 Rai Pal에 별표 표시",
		otherModdersTitle: "다른 모더들",
		otherModdersDescription:
			"Rai Pal은 게임 모딩을 관리하는 데 도움을 주기 위해 만들어졌으며, 다른 개발자가 만든 도구 없이는 할 수 없습니다. 이 사람들 중 일부는 기부 링크가 없지만, 그들의 작업에 매우 감사드립니다.",
		modderOnWebsite: "{modderName}의 {website} 프로필",
		patreonLeaderboard: "Patreon 리더보드",
		rankedByPatreonDonationAmount: "총 기부 금액 순으로 정렬되었습니다.",
		patreonProfilePrivateNotice:
			"여기에서 자신을 볼 수 없는 경우, Patreon 프로필이 비공개로 설정되어 있기 때문입니다.",
	},

	commandButton: {
		cancel: "취소",
		dontAskAgain: "다시 묻지 않기",
	},
} as const;
