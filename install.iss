; Current-user launcher installer. Runtime/config/log are never packaged or deleted.
#ifndef AppVersion
  #define AppVersion "3.0.7"
#endif
#ifndef PackageRoot
  #define PackageRoot "package-root\WarshipPilot"
#endif
[Setup]
AppId={{0D9307FE-8983-488F-9C19-12D18171AF41}
AppName=舰R助手
AppVersion={#AppVersion}
AppPublisher=WarshipPilot
DefaultDirName={localappdata}\WarshipPilot
DefaultGroupName=舰R助手
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
DisableProgramGroupPage=yes
OutputBaseFilename=WarshipPilot-Setup
Compression=lzma2
SolidCompression=yes
SetupIconFile=icons\icon.ico
UninstallDisplayIcon={app}\WarshipPilot.exe
[Languages]
Name: "chinesesimplified"; MessagesFile: "ChineseSimplified.isl"
[Tasks]
Name: desktopicon; Description: "创建桌面快捷方式"; Flags: unchecked
[Files]
Source: "{#PackageRoot}\WarshipPilot.exe"; DestDir: "{app}"; Flags: ignoreversion
[Icons]
Name: "{autoprograms}\舰R助手"; Filename: "{app}\WarshipPilot.exe"; WorkingDir: "{app}"
Name: "{autodesktop}\舰R助手"; Filename: "{app}\WarshipPilot.exe"; WorkingDir: "{app}"; Tasks: desktopicon
[Run]
Filename: "{app}\WarshipPilot.exe"; Description: "启动舰R助手"; WorkingDir: "{app}"; Flags: nowait postinstall skipifsilent
