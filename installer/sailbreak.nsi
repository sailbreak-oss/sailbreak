Unicode true

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "WinMessages.nsh"

!ifndef PRODUCT_VERSION
!define PRODUCT_VERSION "0.1.0"
!endif
!ifndef BIN_DIR
!define BIN_DIR "target\x86_64-pc-windows-msvc\release"
!endif

!define PRODUCT_NAME "Sailbreak"
!define PRODUCT_PUBLISHER "Sailbreak OSS"
!define PRODUCT_UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\Sailbreak"

Name "${PRODUCT_NAME}"
OutFile "dist\sailbreak-v${PRODUCT_VERSION}-windows-x86_64-setup.exe"
InstallDir "$LOCALAPPDATA\Sailbreak"
InstallDirRegKey HKCU "Software\Sailbreak" "InstallDir"
RequestExecutionLevel user

VIProductVersion "${PRODUCT_VERSION}.0"
VIAddVersionKey /LANG=1033 "ProductName" "${PRODUCT_NAME}"
VIAddVersionKey /LANG=1033 "CompanyName" "${PRODUCT_PUBLISHER}"
VIAddVersionKey /LANG=1033 "FileDescription" "Sailbreak hardware control"
VIAddVersionKey /LANG=1033 "FileVersion" "${PRODUCT_VERSION}"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES


Section "Sailbreak" SecSailbreak
    SetShellVarContext current
    SetOutPath "$INSTDIR"
    File "${BIN_DIR}\sailbreak.exe"
    File /oname=sailbreak-cli.exe "${BIN_DIR}\sailbreak.exe"
    File "${BIN_DIR}\sailbreakd.exe"
    File "${BIN_DIR}\sailbreak-gui.exe"

    WriteRegStr HKCU "Software\Sailbreak" "InstallDir" "$INSTDIR"
    WriteRegStr HKCU "${PRODUCT_UNINSTALL_KEY}" "DisplayName" "${PRODUCT_NAME}"
    WriteRegStr HKCU "${PRODUCT_UNINSTALL_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
    WriteRegStr HKCU "${PRODUCT_UNINSTALL_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
    WriteRegStr HKCU "${PRODUCT_UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
    WriteRegStr HKCU "${PRODUCT_UNINSTALL_KEY}" "UninstallString" "$INSTDIR\uninstall.exe"

    Call AddUserPath

    CreateDirectory "$SMPROGRAMS\Sailbreak"
    CreateShortCut "$SMPROGRAMS\Sailbreak\Sailbreak CLI.lnk" "$INSTDIR\sailbreak-cli.exe" "--help"
    CreateShortCut "$SMPROGRAMS\Sailbreak\Daemon status.lnk" "$INSTDIR\sailbreak-cli.exe" "daemon status"
    CreateShortCut "$SMPROGRAMS\Sailbreak\Uninstall Sailbreak.lnk" "$INSTDIR\uninstall.exe"

    WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd
Section "Uninstall"
    SetShellVarContext current
    Call un.RemoveUserPath

    Delete "$SMPROGRAMS\Sailbreak\Sailbreak CLI.lnk"
    Delete "$SMPROGRAMS\Sailbreak\Daemon status.lnk"
    Delete "$SMPROGRAMS\Sailbreak\Uninstall Sailbreak.lnk"
    RMDir "$SMPROGRAMS\Sailbreak"

    DeleteRegKey HKCU "${PRODUCT_UNINSTALL_KEY}"
    DeleteRegKey HKCU "Software\Sailbreak"
    RMDir /r "$INSTDIR"
SectionEnd

Function AddUserPath
    StrCmp $0 "$INSTDIR" AddUserPath_done
    Push $0
    Push "$INSTDIR;"
    Call StrStr
    Pop $1
    StrCmp $1 "" 0 AddUserPath_done

    StrCmp $0 "" 0 AddUserPath_append
    StrCpy $0 "$INSTDIR"
    Goto AddUserPath_write

AddUserPath_append:
    StrCpy $0 "$INSTDIR;$0"

AddUserPath_write:
    WriteRegExpandStr HKCU "Environment" "Path" "$0"
    SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000

AddUserPath_done:
FunctionEnd

Function un.RemoveUserPath
    ReadRegStr $0 HKCU "Environment" "Path"
    StrCmp $0 "$INSTDIR" unRemoveUserPath_clear

    Push $0
    Push "$INSTDIR;"
    Call un.StrStr
    Pop $1
    StrCmp $1 "" unRemoveUserPath_done

    StrLen $2 "$INSTDIR;"
    StrLen $3 $1
    StrCpy $4 $0 -$3
    StrCpy $5 $1 "" $2
    StrCpy $0 "$4$5"
    StrCpy $6 $0 1 -1
    StrCmp $6 ";" 0 +2
        StrCpy $0 $0 -1

    WriteRegExpandStr HKCU "Environment" "Path" "$0"
    SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
    Goto unRemoveUserPath_done

unRemoveUserPath_clear:
    StrCpy $0 ""
    WriteRegExpandStr HKCU "Environment" "Path" "$0"
    SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000

unRemoveUserPath_done:
FunctionEnd

!macro StrStr un
Function ${un}StrStr
    Exch $R1
    Exch
    Exch $R2
    Push $R3
    Push $R4
    Push $R5
    StrLen $R3 $R1
    StrCpy $R4 0

${un}StrStr_loop:
    StrCpy $R5 $R2 $R3 $R4
    StrCmp $R5 $R1 ${un}StrStr_done
    StrCmp $R5 "" ${un}StrStr_done
    IntOp $R4 $R4 + 1
    Goto ${un}StrStr_loop

${un}StrStr_done:
    StrCpy $R1 $R2 "" $R4
    Pop $R5
    Pop $R4
    Pop $R3
    Pop $R2
    Exch $R1
FunctionEnd
!macroend
!insertmacro StrStr ""
!insertmacro StrStr "un."
