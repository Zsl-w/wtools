[Setup]
AppId={{wTools-Desktop-Search}
AppName=wTools
AppVersion=0.4.1
AppPublisher=wTools
AppPublisherURL=https://github.com
DefaultDirName={localappdata}\wTools
DefaultGroupName=wTools
OutputDir=installer_output
OutputBaseFilename=wTools-Setup-0.4.1
Compression=lzma2
SolidCompression=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
SetupIconFile=assets\icon.ico
UninstallDisplayIcon={app}\lib.exe
UninstallDisplayName=wTools

[Files]
Source: "build\windows\x64\runner\Release\*"; DestDir: "{app}"; Flags: recursesubdirs createallsubdirs

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueName: "wTools"; ValueType: string; ValueData: """{app}\lib.exe"""; Flags: uninsdeletevalue

[Icons]
Name: "{group}\wTools"; Filename: "{app}\lib.exe"; IconFilename: "{app}\assets\icon.ico"
Name: "{group}\{cm:UninstallProgram,wTools}"; Filename: "{uninstallexe}"

[Run]
Filename: "{app}\lib.exe"; Description: "启动 wTools"; Flags: nowait postinstall skipifsilent

[Code]
function IsAppRunning(): Boolean;
var
  ResultCode: Integer;
begin
  Result := Exec('cmd.exe', '/C tasklist /FI "IMAGENAME eq lib.exe" | find /I "lib.exe" > nul', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) and (ResultCode = 0);
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
begin
  if CurStep = ssInstall then
  begin
    if IsAppRunning() then
    begin
      Exec('cmd.exe', '/C taskkill /F /IM lib.exe', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
    end;
  end;
end;
