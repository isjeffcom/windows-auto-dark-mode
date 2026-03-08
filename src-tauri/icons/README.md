# App Icons

The app uses the **512px** icon from the project’s `resources/` folder for best quality:

- **Bundle / tray / installer:** `../resources/Tray_Icon_512.ico` (configured in `tauri.conf.json`)

Windows will scale the 512 image down when needed, so Start Menu and taskbar stay sharp. You also have `Tray_Icon_32.ico`, `Tray_Icon_64.ico`, `Tray_Icon_128.ico` and `App_Icon_512.ico` in `resources/` if you want to switch or add more sizes later.

---

## Why the icon can look jagged (Start Menu / taskbar)

Windows uses **multiple icon sizes** for different DPI and contexts. If the `.ico` only contains small sizes (e.g. 32×32, 48×48), Windows **upscales** them for the Start Menu and high-DPI displays, which causes a jagged, pixelated look. Using a **256×256** or **512×512** source avoids that.

## Optional: use a multi-resolution .ico

1. **Prepare a high-resolution source**  
   Use a **256×256** or **512×512** PNG with transparency (e.g. your logo on a transparent background). Draw or export it at that size so edges are smooth and anti-aliased.

2. **Generate a multi-size .ico**  
   Put these sizes into a **single** `.ico` file so Windows can pick the right one:
   - 16×16, 32×32, 48×48 — taskbar, small tiles
   - **96×96, 128×128, 256×256** — Start Menu, high-DPI, crisp display

   Tools you can use:
   - **[IcoFX](https://icofx.ro/)** (Windows) — open your PNG, add/resize to each size, save as .ico
   - **GIMP** — open PNG, Image → Scale to each size, export as .ico (with multiple layers)
   - **Online:** [icoconvert.com](https://icoconvert.com/) or [convertio.co](https://convertio.co/png-ico/) — upload PNG and select “Multiple sizes” / 256×256
   - **ImageMagick** (one 256×256 PNG → multi-size .ico):
     ```bash
     magick convert icon_256.png -define icon:auto-resize=256,128,96,64,48,32,16 icon.ico
     ```

3. **Replace the app icon**  
   Save the new file as `Tray_Icon.ico` in this folder (overwrite the existing one), then rebuild the app. The same file is used for the window, tray, and installer.

## Checklist

- [ ] Source image is at least 256×256 (or 512×512).
- [ ] .ico contains 256×256 (and ideally 128, 96, 48, 32, 16).
- [ ] 32-bit color with alpha (transparency) for smooth edges.
- [ ] Rebuild and reinstall so Windows picks up the new icon (you may need to clear the icon cache or restart Explorer if it doesn’t update).
