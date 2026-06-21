; Glint Installer Script for Inno Setup
; Creates a proper Windows installer that sets Glint as the system default image viewer
; Requires admin privileges to properly register file associations via COM API

#define MyAppName "Glint"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "Samin Yeasar"
#define MyAppURL "https://github.com/solez-ai/glint"
#define MyAppExeName "glint.exe"

[Setup]
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputDir=.
OutputBaseFilename=Glint-Setup-x64
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
; Request admin so we can properly set Glint as default image viewer via COM API
PrivilegesRequired=admin
PrivilegesRequiredOverridesAllowed=dialog
SetupIconFile=assets\glint.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
ShowComponentSizes=no
VersionInfoVersion={#MyAppVersion}
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription={#MyAppName} - Native Windows Photo Viewer
MinVersion=10.0.17763
; Always restart if explorer needs to be refreshed
AlwaysRestart=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"
Name: "setdefault"; Description: "Set Glint as the default photo viewer for all image formats"; GroupDescription: "System integration:"; Flags: checkedonce
Name: "autostart"; Description: "Launch Glint at Windows startup for instant photo viewing"; GroupDescription: "System integration:"; Flags: checkedonce

[Files]
Source: "target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "assets\glint.ico"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "assets\logo.png"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "assets\logo_256.png"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
; 1. First register file associations (HKCU ProgID, OpenWithList, capabilities)
Filename: "{app}\{#MyAppExeName}"; Parameters: "--register-associations"; Flags: runhidden waituntilterminated; Tasks: setdefault
; 2. Then set as actual default via COM API (writes UserChoice hash - needs admin)
Filename: "{app}\{#MyAppExeName}"; Parameters: "--set-default"; Flags: runhidden waituntilterminated; Tasks: setdefault
; 3. Enable auto-start if selected
Filename: "{app}\{#MyAppExeName}"; Parameters: "--enable-autostart"; Flags: runhidden waituntilterminated; Tasks: autostart
; 4. Launch Glint after install
Filename: "{app}\{#MyAppExeName}"; Description: "Launch Glint now"; Flags: nowait postinstall skipifsilent shellexec

[UninstallRun]
Filename: "{app}\{#MyAppExeName}"; Parameters: "--unregister-associations"; Flags: runhidden waituntilterminated
Filename: "{app}\{#MyAppExeName}"; Parameters: "--disable-autostart"; Flags: runhidden waituntilterminated

[Registry]
; ProgID (also done by --register-associations, but here for redundancy)
Root: HKCU; Subkey: "Software\Classes\Glint.ImageViewer"; ValueType: string; ValueName: ""; ValueData: "Glint Image Viewer"; Flags: uninsdeletekey; Tasks: setdefault
Root: HKCU; Subkey: "Software\Classes\Glint.ImageViewer\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},1"; Tasks: setdefault
Root: HKCU; Subkey: "Software\Classes\Glint.ImageViewer\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: setdefault
Root: HKCU; Subkey: "Software\Classes\Glint.ImageViewer\shell\edit\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: setdefault

; Auto-start
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "Glint"; ValueData: """{app}\{#MyAppExeName}"" --background"; Tasks: autostart; Flags: uninsdeletevalue
