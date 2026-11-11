# ⚡ Finder

> **Next-generation, ultra-fast, modular system search launcher written in Rust.**  
> Inspired by Raycast, macOS Spotlight, and Rofi. Designed for Windows 10/11 and Arch Linux (Wayland & X11) with minimal memory footprint and zero CPU overhead when idle.

---

## 🚀 Key Features

* **Glassmorphism & Pitch Black (#000000)**: Hardware-accelerated frosted glass backdrop blur, anti-aliased smooth rounded corners (DWM `DWMWA_WINDOW_CORNER_PREFERENCE = DWMWCP_ROUND` on Windows 11, clean alpha compositing on Wayland/X11).
* **Smart Bottom-Center Window Placement**: Automatically docks horizontally centered slightly above the screen's bottom border (`y = monitor_height - window_height - offset`), giving a modern HUD-like experience.
* **Global Activation Hotkey**: Default toggle on **`Right Alt`** (or rebindable to `Alt+Space`, `Super+Space` in `config.toml`).
* **Zero Idle CPU Resource Monitor**: Real-time status bar showing CPU load & clock frequency, RAM usage, and GPU utilization/VRAM (NVML for NVIDIA, sysfs for AMD/Intel). Automatically enters a zero-cost sleep when the launcher is closed.
* **Instant Application Launcher**:
  * *Windows*: Scans Start Menu (`%ProgramData%`, `%APPDATA%`), system utilities (`taskmgr`, `cmd`, `powershell`, `control`, etc.), and `%PATH%`.
  * *Linux*: Scans `.desktop` entries in `/usr/share/applications` and `~/.local/share/applications`, and `$PATH`.
* **Multi-Threaded File Search**:
  * Crawls user directories with automatic exclusion of cache & build artifacts (`node_modules`, `.git`, `target`, `vendor`, `Windows`, `proc`, `sys`).
  * Instant keystroke cancellation via Tokio `CancellationToken` to avoid wasted work.
* **Embedded Math & Unit Converter**:
  * Mathematical evaluation powered by `evalexpr` (`sin(45)`, `2^16`, `15 * 144`, `sqrt(256)`).
  * Unit and currency conversions (`100 km to mi`, `1024 mb to gb`, `50 c to f`, `10 kg to lbs`).
* **AI Search & Local LLM**:
  * Web shortcuts to Google AI Overview, Google Gemini, and ChatGPT.
  * Native local LLM execution via Ollama API (`/ai <prompt>`) displayed directly in the launcher window without opening a browser.
* **Scratchpad & Notes**:
  * Save instant notes with `/note <text>`.
  * View recent notes with `/notes` stored locally in markdown.
* **System Commands**:
  * Control workstation: `lock`, `sleep`, `restart`, `shutdown`, `empty trash`, `kill <process_name>`.

---

## 🏗️ Architecture

```
omni-launcher/
├── Cargo.toml                  # Workspace manifest
├── ui/                         # Lightweight hardware-accelerated Webview UI
│   ├── index.html              # HTML5 launcher HUD layout
│   ├── style.css               # Pitch black OLED & glassmorphism styling
│   └── app.js                  # Event coordinator & keyboard navigation
└── src-tauri/
    ├── Cargo.toml              # Rust crate dependencies
    ├── build.rs                # Tauri build orchestrator
    ├── tauri.conf.json         # Window geometry, transparency, capabilities
    ├── capabilities/
    │   └── default.json        # Tauri v2 security ACL
    └── src/
        ├── main.rs & lib.rs    # Tauri runtime initialization & IPC handlers
        ├── config.rs           # TOML configuration loader/writer
        ├── window.rs           # Bottom-center positioning & DWM/vibrancy effects
        ├── hotkey.rs           # Global shortcut listener (Right Alt & combo keys)
        ├── monitor.rs          # Real-time CPU, RAM, GPU sysinfo worker
        ├── system.rs           # Workstation lock/suspend/kill/power commands
        └── search/
            ├── mod.rs          # Unified search coordinator & scoring
            ├── apps.rs         # Windows .lnk & Linux .desktop indexing
            ├── files.rs        # Multi-threaded file crawler with cancel token
            ├── calc.rs         # Arithmetic expression evaluator & converter
            ├── web.rs          # Google & DuckDuckGo live autocomplete
            ├── ai.rs           # Ollama local LLM integration & web AI
            └── notes.rs        # Scratchpad markdown notes storage
```

---

## 🛠️ Build & Installation

### Windows 10 / 11
1. Ensure Rust is installed:
   ```powershell
   rustup default stable-x86_64-pc-windows-msvc
   ```
2. Build the release binary:
   ```powershell
   npm install
   npm run tauri build
   ```
   Or build directly via cargo:
   ```powershell
   cd src-tauri
   cargo build --release
   ```

### Arch Linux (Wayland / X11)
1. Install system prerequisites:
   ```bash
   sudo pacman -S --needed base-devel webkit2gtk-4.1 openssl gtk3 libayatana-appindicator
   ```
2. Compile and package:
   ```bash
   npm install
   npm run tauri build
   ```

---

## ⚙️ Configuration (`config.toml`)

The configuration file is automatically generated at:
* **Windows**: `%APPDATA%\OmniLauncher\config.toml`
* **Linux**: `~/.config/omni-launcher/config.toml`

```toml
hotkey = "RightAlt"                # e.g., "RightAlt", "Alt+Space", "Ctrl+Space"
bottom_offset = 80                 # Pixels above bottom edge
theme = "oled"                     # "oled", "glass", "dark"
hide_on_blur = true                # Automatically hide when focus is lost
ollama_model = "auto"              # e.g., "auto", "llama3", "mistral", "phi3"
file_search_dirs = [
    "C:\\Users\\User\\Documents",
    "C:\\Users\\User\\Downloads",
    "C:\\Users\\User\\Desktop"
]
file_exclude_patterns = [
    "node_modules",
    ".git",
    "target",
    "vendor",
    "Windows",
    "$Recycle.Bin"
]
```

---

## 📄 License

MIT OR Apache-2.0 © Hudie125
