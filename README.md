![Alt text](readme_cover.png)
# Auto Dark Mode

A lightweight Windows app that automatically switches between dark and light mode by **fixed time** or **sunrise/sunset** (location-based). Built with [Tauri 2](https://v2.tauri.app/) and React.

## Features

- **Schedule**: Fixed times (e.g. 07:00 → Light, 19:00 → Dark) or sunrise/sunset by latitude/longitude
- **System theme**: Switch taskbar and window borders
- **App theme**: Switch app title bar (e.g. Explorer, Settings)
- **System tray**: Run in background; close window to minimize to tray
- **Start with Windows**: Optional autostart
- **Languages**: English and 中文

## Requirements

- **Windows 10/11** (64-bit)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually preinstalled on Windows 11)

## Download (GitHub Releases)

- **Installer** (`Auto Dark Mode_[VERSION]_x64-setup.exe`): Recommended. Run to install and optionally add a Start Menu shortcut and “Start with Windows”.
- **Portable** (`windows-auto-dark-mode.exe`): Single executable; no install. Place anywhere and run.

## Build from source

### Prerequisites

- [Node.js](https://nodejs.org/) (LTS)
- [Rust](https://rustup.rs/)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (Windows: “Desktop development with C++”)
- [NSIS](https://nsis.sourceforge.io/Download) (for creating the installer)

### Commands

```bash
# Install dependencies
npm install

# Development (with hot reload)
npm run tauri:dev

# Build for release (installer + portable exe)
npm run tauri:build
```

### Build output

After `npm run tauri:build`:

| Artifact | Path |
|----------|------|
| **Installer** | `src-tauri/target/release/bundle/nsis/Auto Dark Mode_[VERSION]_x64-setup.exe` |
| **Portable exe** | `src-tauri/target/release/windows-auto-dark-mode.exe` |

Use these for **GitHub Releases**: upload the setup exe and (optionally) the portable exe or a zip containing it.

### Publishing to GitHub Release

- **Option A (CI):** Push a tag (e.g. `git tag v1.0.0 && git push origin v1.0.0`). The [Release workflow](.github/workflows/release.yml) builds on Windows and uploads **installer** and **portable** as artifacts. Open the workflow run → Artifacts, download both, then create a new Release and attach the files.
- **Option B (local):** Run `npm run tauri:build` on Windows, then upload the two paths above to a new Release.

## Usage

1. Run the app (from Start Menu or the portable exe).
2. **Dashboard**: See current theme and next switch time; optionally switch theme manually.
3. **Schedule**: Choose “Fixed time” or “Sunrise & sunset (location)”; set times or coordinates.
4. **Settings**: Toggle system/app theme switching, tray icon, “Start with Windows”, and language (EN/中文).
5. Close the window to minimize to the system tray; right‑click the tray icon for **Show** or **Quit**.

## License

MIT
