!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "x64.nsh"
!include "LogicLib.nsh"
!include "Win\RestartManager.nsh"

!define APP_NAME "Rai Pal"
!define APP_EXE "Rai-Pal.exe"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"

Name "${APP_NAME}"
OutFile "${OUTPUT_FILE}"
InstallDir "$PROGRAMFILES64\${APP_NAME}"
InstallDirRegKey HKLM "${UNINSTALL_KEY}" "InstallLocation"
RequestExecutionLevel admin
Unicode true
SetCompressor /SOLID lzma

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "${APP_NAME}"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "FileDescription" "${APP_NAME} installer"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "LegalCopyright" "Raicuparta"

!define MUI_ABORTWARNING
!define MUI_ICON "${ICON_FILE}"
!define MUI_UNICON "${ICON_FILE}"
!define MUI_WELCOMEFINISHPAGE_BITMAP "${SIDEBAR_FILE}"
!define MUI_UNWELCOMEFINISHPAGE_BITMAP "${SIDEBAR_FILE}"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Arabic"
!insertmacro MUI_LANGUAGE "Bulgarian"
!insertmacro MUI_LANGUAGE "Dutch"
!insertmacro MUI_LANGUAGE "French"
!insertmacro MUI_LANGUAGE "German"
!insertmacro MUI_LANGUAGE "Italian"
!insertmacro MUI_LANGUAGE "Japanese"
!insertmacro MUI_LANGUAGE "Korean"
!insertmacro MUI_LANGUAGE "Portuguese"
!insertmacro MUI_LANGUAGE "PortugueseBR"
!insertmacro MUI_LANGUAGE "Russian"
!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_LANGUAGE "Spanish"
!insertmacro MUI_LANGUAGE "SpanishInternational"
!insertmacro MUI_LANGUAGE "Swedish"
!insertmacro MUI_LANGUAGE "TradChinese"
!insertmacro MUI_LANGUAGE "Turkish"
!insertmacro MUI_LANGUAGE "Ukrainian"

Function .onInit
	${GetParameters} $R0
	${GetOptions} $R0 "/P" $R1
	IfErrors passiveDone
	SetSilent silent
	passiveDone:
FunctionEnd

Section "Install"
	SetOutPath "$INSTDIR"

	!insertmacro RestartManager_StartSession $0
	${If} $0 != ""
		!insertmacro RestartManager_RegisterFile $0 "$INSTDIR\${APP_EXE}"
		!insertmacro RestartManager_Shutdown $0
		!insertmacro RestartManager_EndSession $0
	${EndIf}

	File "${SOURCE_EXE}"
	File "${ICON_FILE}"
	WriteUninstaller "$INSTDIR\Uninstall.exe"

	CreateDirectory "$SMPROGRAMS\${APP_NAME}"
	CreateShortcut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}" "" "$INSTDIR\icon.ico"
	CreateShortcut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}" "" "$INSTDIR\icon.ico"

	WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayName" "${APP_NAME}"
	WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayVersion" "${VERSION}"
	WriteRegStr HKLM "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
	WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\icon.ico"
	WriteRegStr HKLM "${UNINSTALL_KEY}" "Publisher" "Raicuparta"
	WriteRegStr HKLM "${UNINSTALL_KEY}" "UninstallString" "$INSTDIR\Uninstall.exe"
	WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoModify" 1
	WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoRepair" 1

	${GetParameters} $R0
	${GetOptions} $R0 "/UPDATER" $R1
	IfErrors checkRestart
	Goto relaunchApp
	checkRestart:
	${GetOptions} $R0 "/R" $R1
	IfErrors updaterDone
	relaunchApp:
	ExecShell "open" "$WINDIR\explorer.exe" '"$INSTDIR\${APP_EXE}"'
	updaterDone:
SectionEnd

Section "Uninstall"
	Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
	RMDir "$SMPROGRAMS\${APP_NAME}"
	Delete "$DESKTOP\${APP_NAME}.lnk"
	Delete "$INSTDIR\${APP_EXE}"
	Delete "$INSTDIR\icon.ico"
	Delete "$INSTDIR\Uninstall.exe"
	RMDir "$INSTDIR"
	DeleteRegKey HKLM "${UNINSTALL_KEY}"
SectionEnd
