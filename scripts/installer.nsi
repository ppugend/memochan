; Define defaults if not provided
!ifndef APP_NAME
  !define APP_NAME "MemoChan"
!endif
!ifndef VERSION
  !define VERSION "0.0.0"
!endif
!ifndef PUBLISHER
  !define PUBLISHER "Ppugend"
!endif
!ifndef EXE_NAME
  !define EXE_NAME "memochan.exe"
!endif
!ifndef EXE_PATH
  !define EXE_PATH "target/x86_64-pc-windows-gnu/release/memochan.exe"
!endif
!ifndef OUTPUT_DIR
  !define OUTPUT_DIR "target/x86_64-pc-windows-gnu/release/bundle"
!endif
!ifndef LICENSE_PATH
  !define LICENSE_PATH "LICENSE"
!endif

!ifndef ARCH
  !define ARCH "amd64"
!endif

Name "${APP_NAME}"
OutFile "${OUTPUT_DIR}/${APP_NAME}_Setup_${VERSION}_${ARCH}.exe"
InstallDir "$PROGRAMFILES64\${APP_NAME}"
InstallDirRegKey HKCU "Software\${APP_NAME}" ""

RequestExecutionLevel admin

Section "MemoChan (required)"
  SectionIn RO
  SetOutPath "$INSTDIR"
  
  ; Install executable
  File /nonfatal "${EXE_PATH}"
  
  ; Install license if exists
  IfFileExists "${LICENSE_PATH}" 0 +2
    File "${LICENSE_PATH}"
  
  ; Write uninstaller
  WriteUninstaller "$INSTDIR\uninstall.exe"
  
  ; Registry keys for Add/Remove Programs
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "Publisher" "${PUBLISHER}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayVersion" "${VERSION}"
  
  ; Start Menu Shortcut
  CreateDirectory "$SMPROGRAMS\${APP_NAME}"
  CreateShortcut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${EXE_NAME}"
  CreateShortcut "$SMPROGRAMS\${APP_NAME}\Uninstall.lnk" "$INSTDIR\uninstall.exe"
  
  ; Desktop Shortcut
  CreateShortcut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${EXE_NAME}"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\${EXE_NAME}"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\uninstall.exe"
  
  Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
  Delete "$SMPROGRAMS\${APP_NAME}\Uninstall.lnk"
  RMDir "$SMPROGRAMS\${APP_NAME}"
  
  Delete "$DESKTOP\${APP_NAME}.lnk"
  
  RMDir "$INSTDIR"
  
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"
  DeleteRegKey HKCU "Software\${APP_NAME}"
SectionEnd
