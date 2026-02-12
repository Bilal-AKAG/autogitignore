param(
    [string]$Version
)

$ErrorActionPreference = "Stop"
$Repo = "Bilal-AKAG/autogitignore"
$BinName = "autogitignore.exe"

if (-not $Version) {
    $latest = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    $Version = $latest.tag_name
}

if (-not $Version) {
    throw "Could not determine release version."
}

$target = "x86_64-pc-windows-msvc"
$archive = "autogitignore-$Version-$target.zip"
$url = "https://github.com/$Repo/releases/download/$Version/$archive"

$tmpDir = New-Item -ItemType Directory -Force -Path ([System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid().ToString())
$zipPath = Join-Path $tmpDir.FullName $archive

Invoke-WebRequest -Uri $url -OutFile $zipPath
Expand-Archive -Path $zipPath -DestinationPath $tmpDir.FullName -Force

$installDir = if ($env:BINDIR) { $env:BINDIR } else { Join-Path $HOME ".local\\bin" }
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Copy-Item (Join-Path $tmpDir.FullName $BinName) (Join-Path $installDir $BinName) -Force

$line = "========================================"
$oldStyle = $Host.Name -eq "ConsoleHost" -and $PSVersionTable.PSVersion.Major -lt 7

Write-Host ""
if ($oldStyle) {
    Write-Host $line
    Write-Host "  Thanks for installing autogitignore"
    Write-Host $line
    Write-Host ""
    Write-Host "Installed version: $Version"
    Write-Host "Binary path: $(Join-Path $installDir $BinName)"
    Write-Host ""
    Write-Host "Try it now:"
    Write-Host "  autogitignore"
    Write-Host ""
    Write-Host "If command is not found, add this to PATH:"
    Write-Host "  $installDir"
    Write-Host ""
} else {
    Write-Host $line -ForegroundColor Cyan
    Write-Host "  Thanks for installing autogitignore" -ForegroundColor Green
    Write-Host $line -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Installed version: " -NoNewline
    Write-Host $Version -ForegroundColor White
    Write-Host "Binary path: " -NoNewline
    Write-Host (Join-Path $installDir $BinName) -ForegroundColor White
    Write-Host ""
    Write-Host "Try it now:" -ForegroundColor Yellow
    Write-Host "  autogitignore" -ForegroundColor Green
    Write-Host ""
    Write-Host "If command is not found, add this to PATH:" -ForegroundColor Yellow
    Write-Host "  $installDir" -ForegroundColor White
    Write-Host ""
}
