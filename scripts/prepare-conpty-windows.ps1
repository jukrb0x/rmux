# Stages Microsoft's paired runtime for developer tests and Windows packages.
param(
    [Parameter(Mandatory)][ValidateNotNullOrEmpty()][string[]]$Destination,
    [ValidateSet("x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc")]
    [string]$Target = "x86_64-pc-windows-msvc"
)
$ErrorActionPreference = "Stop"
$version = "1.24.261001001"
$expected = "4d6aaddc1d2385c9f5897df28f33879f699f8f2783315d5204cf3d8c3616ac5f"
$cache = Join-Path (Split-Path -Parent $PSScriptRoot) "target/conpty-$version"
New-Item -ItemType Directory -Force -Path $cache | Out-Null
$archive = Join-Path $cache "conpty.zip"
if (-not (Test-Path -LiteralPath $archive)) {
    Invoke-WebRequest "https://api.nuget.org/v3-flatcontainer/microsoft.windows.console.conpty/$version/microsoft.windows.console.conpty.$version.nupkg" -OutFile $archive
}
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expected) {
    throw "Microsoft ConPTY $version archive checksum mismatch: $archive"
}
$package = Join-Path $cache "package"
Expand-Archive -LiteralPath $archive -DestinationPath $package -Force
$architecture = if ($Target -eq "aarch64-pc-windows-msvc") { "arm64" } else { "x64" }
$runtime = @(
    Join-Path $package "runtimes/win-$architecture/native/conpty.dll"
    Join-Path $package "build/native/runtimes/$architecture/OpenConsole.exe"
)
foreach ($file in $runtime) {
    $signature = Get-AuthenticodeSignature -LiteralPath $file
    if ($signature.Status -ne "Valid" -or $signature.SignerCertificate.Subject -notmatch 'CN=Microsoft Corporation,') {
        throw "Microsoft ConPTY signature is invalid: $file"
    }
}
foreach ($directory in $Destination) {
    New-Item -ItemType Directory -Force -Path $directory | Out-Null
    foreach ($file in $runtime) {
        Copy-Item -LiteralPath $file -Destination $directory -Force
    }
}
