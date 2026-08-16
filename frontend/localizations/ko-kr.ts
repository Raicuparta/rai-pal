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

	gamesPage: {
		emptyGamesList:
			"Rai Pal이 게임을 하나도 못 찾았어요. Steam이나 Epic 같은 게임 플랫폼에서 설치된 앱이 있어야 해요.",
		emptyFilteredGamesList:
			"아무것도 없어요! 선택한 필터 때문에 게임이 전부 숨겨졌어요. 필터 지우면 다시 보여요.",
		emptyGamesLoading: "게임을 찾는 중...",
	},

	manualGames: {
		manualSteamSupportNote:
			"참고: Steam이 아닌 게임을 Steam에 추가해도 Rai Pal에서 계속 감지됩니다.",
		savedDirectories:
			"Rai Pal이 데이터를 새로고침할 때마다 이 디렉토리들이 재귀적으로 스캔됩니다. 큰 폴더는 속도를 현저히 느리게 할 수 있습니다.",
		button: "게임 추가",
		title: "수동으로 추가된 게임",
		selectGameExecutable: "게임 실행 파일 선택",
		selectGamesDirectory: "게임 디렉토리 선택. 느릴 수 있음!",
		fileDropNote:
			"게임 실행 파일이나 폴더 전체를 Rai Pal 창 아무 곳에나 끌어다 놓으면 이 대화상자를 열지 않고도 라이브러리에 추가할 수 있습니다.",
		scanning: "{path} 스캔 중...",
		scanProgress:
			"{directories}개 디렉토리 스캔, {executables}개 실행 파일 발견",
		scanComplete:
			"{duration}초 만에 {gamesCount}개 실행 파일을 찾았습니다. 이 폴더를 Rai Pal에 추가할까요?",
		confirmAddFolder: "폴더 추가",
		cancel: "취소",
	},

	refresh: {
		button: "새로고침",
		buttonUpdateRemoteDatabases: "원격 데이터베이스 업데이트",
		loading: "{items} 로딩 중...",
	},

	filterMenu: {
		button: "필터",
		resetButton: "재설정",
		searchPlaceholder: "검색...",
	},

	filterProperty: {
		provider: "플랫폼",
		tags: "태그",
		architecture: "아키텍처",
		os: "운영체제",
		unityBackend: "Unity 백엔드",
		engine: "엔진",
		status: "상태",
		mod: "모드",
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
		providerXboxOnlyInstalled: "설치된 경우에만 PC Xbox 게임을 보여줍니다.",
		engineGodotNotFullySupported: "Godot 게임은 아직 완전히 지원되지 않습니다.",
		engineGameMakerNotFullySupported:
			"GameMaker 게임은 아직 완전히 지원되지 않습니다.",
	},

	providerCommand: {
		installGame: "설치",
		showGameInLibrary: "라이브러리에서 보기",
		showGameInStore: "스토어 페이지 열기",
		startGameViaProvider: "게임 시작",
		startGameViaExe: "게임 실행 파일 실행",
		openGamePageInBrowser: "브라우저에서 열기",
	},

	gameModal: {
		startGameButton: "게임 시작",
		startGameExecutable: "실행 파일로 게임 시작",
		startGameViaProvider: "{provider}로 게임 시작",
		foldersDropdown: "폴더",
		openGameFilesFolder: "게임 파일 폴더 열기",
		openInstalledModsFolder: "설치된 모드 폴더 열기",
		openGameDataFolder: "게임 앱 데이터 폴더 열기",
		openGameWinePrefixFolder: "게임 Wine 프리픽스 폴더 열기",
		openGameWineBinaryFolder: "게임 Wine 바이너리 폴더 열기",
		removeFromRaiPal: "Rai Pal에서 제거",
		removeGameConfirmation: "이 게임을 Rai Pal에서 제거할까요?",
		refreshGame: "새로고침",
		failedToReadGameInfo:
			"이 게임의 중요 정보를 읽을 수 없었어요. 실행 파일이 보호돼 있어서 그럴 수 있어요. 일부 모드가 설치 안 될 수도 있어요.",
		failedToDetermineEngine:
			"이 게임의 엔진을 확인할 수 없었어요. 일부 모드가 설치 안 될 수도 있어요.",
		gameModsLabel: "모드",
		gameNotInstalledWarning:
			"이 게임이 설치되지 않아서 어떤 모드가 맞는지 100% 장담할 수 없어요. 아래 모드들이 될 수도 있어요. 게임을 설치하면 더 정확히 알려드릴게요.",
		uninstallAllModsButton: "모드 모두 제거",
		uninstallAllModsConfirmation:
			"확실해요? 이 게임 모드 폴더 안 파일이 전부 지워져요. 실제 게임 파일은 그대로 두고요.",

		incompatibleGameModsLabel: "호환되지 않는 모드",

		incompatibleGameModsDescription:
			"여기 모드들은 이 게임 엔진 버전과 호환이 안 돼서 설치할 수 없어요.",

		otherThings: "기타 항목",

		otherThingsDescription:
			"이것들은 대부분 종속성 같은 거라서 보통 직접 건드릴 일이 없어요.",
	},

	gameModRow: {
		editModConfig: "모드 설정 편집",
		openModConfigFolderTooltip: "이 모드의 설정 파일이 포함된 폴더 열기",
		openModFolder: "모드 폴더 열기",
		updateMod: "업데이트",
		installMod: "설치",
		installModAnticheatWarning:
			"멀티플레이 게임에 모드 설치할 땐 조심하세요! 안티치트가 모드를 감지하면 해롭지 않아 보여도 밴 당할 수 있어요.",
		reinstallMod: "재설치",
		uninstallMod: "제거",
		runMod: "실행",
		downloadRemoteConfig: "권장 설정 다운로드",
		remoteConfigAvailable:
			"권장 설정이 있어요. 아직 설정이 없으면 자동으로 받고, 세 점 메뉴에서 강제로 받을 수도 있어요.",
		modOutdated: "업데이트 가능",
		cantUninstallModWithDependants:
			"이 모드를 쓰는 다른 모드가 있으면 제거할 수 없어요. 그 모드들부터 먼저 제거해 주세요.",
	},

	gamesTableColumn: {
		game: "게임",
		engine: "엔진",
		date: "날짜",
	},

	modsPage: {
		openLocalModsFolderButton: "모드 폴더 열기",
		openLoadlModsFolderTooltip:
			"rai-pal-manifest.json이 있는 모드를 이 폴더에 넣으면 온라인 데이터베이스 없이 바로 불러올 수 있어요.",
		tableColumnMod: "모드",
		tableColumnVersion: "버전",
		tableColumnGameEngine: "엔진",
		tableColumnUnityBackend: "백엔드",
		modByAuthor: "{authorName} 제작",
		modDeprecated: "사용 중지됨",
		modDeprecatedTooltip:
			"이 모드는 더 이상 지원되지 않아요. 제거하고 더 최신 모드를 설치하는 게 좋아요.",
	},

	modModal: {
		runMod: "실행",
		openModFolder: "모드 폴더 열기",
		updateMod: "모드 업데이트",
		downloadMod: "모드 다운로드",
		deleteMod: "모드 삭제",
		deleteModConfirmation: "확실해요? 모드 폴더 안 파일이 전부 날아가요.",
		byAuthor: "{authorName} 제작",
	},

	appDropdownMenu: {
		showGameThumbnails: "목록에 게임 썸네일 표시",
		language: "언어",
		autoDetectedLanguage: "자동 감지 - {languageName}",
		resetRaiPalSettingsButton: "Rai Pal 설정 재설정",
		resetRaiPalSettingsTooltip: "필터, 확인 창, 이것저것 다 초기화돼요.",
		openLogsFolderButton: "로그 폴더 열기",
		clearRaiPalCacheOpenModal: "Rai Pal 캐시 지우기",
		clearRaiPalCacheTooltip: "Rai Pal이 사용하는 게임 목록 캐시를 지웁니다.",
	},

	userMenu: {
		unknownUser: "알 수 없는 사용자",
		logOut: "로그아웃",
		signInWithDiscord: "Discord로 로그인",
		discordAccessNote:
			"모드가 이걸로 Discord 사용자 이름, 아바타, 역할 등에 접근할 수 있어요.",
	},

	subPage: {
		back: "뒤로",
	},

	downloadStatusMenu: {
		clear: "완료 항목 지우기",
	},

	steamCache: {
		resetSteamCacheButton: "Steam 캐시 재설정",
		resetSteamCacheModalTitle: "Steam 캐시 재설정",
		resetSteamCacheDescription:
			"Rai Pal에 내가 가지고 있지 않은 Steam 게임이 표시될 때 쓰세요. Steam 캐시를 초기화한 다음 Steam을 다시 시작해야 해요. 이미 지워진 파일이면 오류가 나요.",
		resetSteamCacheSuccess:
			"캐시 파일이 삭제되었습니다. Steam을 다시 시작하고 몇 초 기다린 후 Rai Pal에서 새로고침 버튼을 누르세요.",
	},

	steamShortcut: {
		addRaiPalSteamShortcutButton: "Rai Pal을 Steam 라이브러리에 추가",
		addRaiPalSteamShortcutModalTitle: "Rai Pal을 Steam 라이브러리에 추가",
		addRaiPalSteamShortcutDescription:
			"이건 특히 Steam Deck에서 게임 모드로 Rai Pal을 실행할 때 좋아요. 하고 나서 Steam을 다시 시작해야 해요.",
		addRaiPalSteamShortcutSuccess:
			"Rai Pal이 Steam 라이브러리에 추가되었습니다. 보려면 Steam을 다시 시작하세요.",
	},

	globalWineOverrides: {
		setUpEnvironmentButton: "BepInEx용 Linux 환경 설정",
		setUpEnvironmentTitle: "BepInEx용 Linux 환경 설정",
		setUpEnvironmentDescription:
			"Linux에서 Proton/Wine을 쓸 때, Wine 설정을 안 해두면 BepInEx가 자동으로 로드되지 않아요. 이 버튼은 WINEDLLOVERRIDES 환경 변수를 전역으로 'winhttp.dll=n,b'로 설정해요. 직접 하고 싶으면 누르지 마세요.",
		setUpEnvironmentSuccess:
			"파일이 생성됐어요. 적용하려면 로그아웃 후 다시 로그인하거나 컴퓨터를 다시 시작해야 해요.",
	},

	debugData: {
		debugDataTitle: "디버그 데이터",
		debugDataCopy: "디버그 데이터 복사",
	},

	thanksPage: {
		intro:
			"안녕하세요. Rai Pal을 만든 사람이에요. 예전에 여러 VR 모드도 만들었고, 지금은 Unity 게임용 범용 VR 모드를 만들고 있어요. 제 작업이 마음에 들고 더 보고 싶으시면 후원 고려해 주세요! itch.io에서 제 무료 모드를 구매해서 응원해 주셔도 좋아요.",
		starRaiPalOnGitHub: "GitHub에서 Rai Pal 별표 주기",
		otherModdersTitle: "다른 모더들",
		otherModdersDescription:
			"Rai Pal은 게임 모딩을 쉽게 관리하도록 도와주는 도구예요. 다른 개발자분들이 만든 도구 없이는 불가능했을 거예요. 어떤 분들은 후원 링크가 없지만, 그래도 그분들 작업에 정말 감사드려요.",
		modderOnWebsite: "{website}의 {modderName}",
		patreonLeaderboard: "Patreon 리더보드",
		rankedByPatreonDonationAmount: "총 후원 금액 기준으로 정렬됐어요.",
		patreonProfilePrivateNotice:
			"여기에 안 보이시면 Patreon 프로필이 비공개라서 그래요.",
	},

	commandButton: {
		cancel: "취소",
		dontAskAgain: "다시 묻지 않기",
	},

	urlModSources: {
		title: "모드 소스",
		add: "소스 추가",
		addSourceDescription:
			"Rai Pal이 기본 데이터베이스 외에도 여기에 제공된 모든 데이터베이스에서 모드를 확인합니다. 딥링크 {deepLink} 를 통해 모드 소스를 공유할 수 있습니다.",
		confirmModalTitle: "모드 소스 확인",
		modsFound: "이 소스에서 {count}개의 모드를 찾았습니다",
		addSource: "소스 추가",
		cancel: "취소",
		loading: "모드 로딩 중...",
	},
} as const;
