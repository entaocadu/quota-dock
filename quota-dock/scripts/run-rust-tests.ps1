param([string]$ExistingExecutable)
$ErrorActionPreference = 'Stop'
$project = Split-Path $PSScriptRoot -Parent
$manifest = @'
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency><dependentAssembly><assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*" /></dependentAssembly></dependency>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3"><security><requestedPrivileges><requestedExecutionLevel level="asInvoker" uiAccess="false" /></requestedPrivileges></security></trustInfo>
  <application xmlns="urn:schemas-microsoft-com:asm.v3"><windowsSettings><longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware></windowsSettings></application>
</assembly>
'@
if (-not ('QuotaDockTestResource' -as [type])) {
  Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class QuotaDockTestResource {
  [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)] public static extern IntPtr BeginUpdateResource(string path, bool delete);
  [DllImport("kernel32.dll", SetLastError=true)] public static extern bool UpdateResource(IntPtr handle, IntPtr type, IntPtr name, ushort language, byte[] data, uint size);
  [DllImport("kernel32.dll", SetLastError=true)] public static extern bool EndUpdateResource(IntPtr handle, bool discard);
}
'@
}
if ($ExistingExecutable) { $executables = @((Resolve-Path -LiteralPath $ExistingExecutable).Path) }
else {
  Push-Location (Join-Path $project 'src-tauri')
  try { $lines = & cargo test --no-run --message-format=json; $buildExit = $LASTEXITCODE; $lines | ForEach-Object { try { $item = $_ | ConvertFrom-Json; if ($item.reason -eq 'compiler-message') { Write-Host $item.message.rendered } } catch {} }; if ($buildExit -ne 0) { throw 'Rust test compilation failed' } }
  finally { Pop-Location }
  $executables = @($lines | ForEach-Object { try { $item = $_ | ConvertFrom-Json; if ($item.reason -eq 'compiler-artifact' -and $item.profile.test -and $item.executable) { $item.executable } } catch {} })
}
if ($executables.Count -eq 0) { throw 'No test executables found' }
foreach ($executable in $executables) {
  $resolved = [IO.Path]::GetFullPath($executable)
  $target = [IO.Path]::GetFullPath((Join-Path $project 'src-tauri/target')) + [IO.Path]::DirectorySeparatorChar
  if (-not $resolved.StartsWith($target, [StringComparison]::OrdinalIgnoreCase)) { throw 'Test executable must be inside the project target directory' }
  # Rust's GNU test harness omits Common Controls v6; Tauri's TaskDialogIndirect import requires it.
  $bytes = [Text.Encoding]::UTF8.GetBytes($manifest)
  $handle = [QuotaDockTestResource]::BeginUpdateResource($resolved, $false)
  if ($handle -eq [IntPtr]::Zero) { throw 'Cannot open test resources' }
  if (-not [QuotaDockTestResource]::UpdateResource($handle, [IntPtr]24, [IntPtr]1, 1033, $bytes, $bytes.Length)) { [QuotaDockTestResource]::EndUpdateResource($handle, $true) | Out-Null; throw 'Cannot embed test manifest' }
  if (-not [QuotaDockTestResource]::EndUpdateResource($handle, $false)) { throw 'Cannot save test manifest' }
  $profile = if ($resolved -match '[\\/]release[\\/]') { 'release' } else { 'debug' }
  $loader = Join-Path $target "$profile/WebView2Loader.dll"
  if (Test-Path -LiteralPath $loader) { Copy-Item -LiteralPath $loader -Destination (Join-Path (Split-Path $resolved) 'WebView2Loader.dll') -Force }
  & $resolved
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
