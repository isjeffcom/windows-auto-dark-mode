# Ensure Cargo is on PATH (for Cursor/terminals that don't load user PATH)
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path $cargoBin) {
  $env:Path = "$cargoBin;$env:Path"
}
& npx tauri dev
