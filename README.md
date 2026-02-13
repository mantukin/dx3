<div align="center">
  <img src="src-tauri/icons/app_icon.png" width="128" height="128" alt="Dx3 Icon" />
  <h1>Dx3 Controller 🎮</h1>
  
  <p>
    <img src="https://img.shields.io/badge/version-1.0.0-blue" alt="Version" />
    <img src="https://img.shields.io/badge/Built_with-Rust_🦀-orange" alt="Rust" />
    <img src="https://img.shields.io/badge/Framework-Tauri-blueviolet" alt="Tauri" />
    <img src="https://img.shields.io/badge/Platform-Windows-0078D6" alt="Windows" />
  </p>
  
  <p>
    <b>Lightweight. Native. Powerful.</b>
  </p>
</div>

**Dx3 Controller** is a next-generation utility designed to seamlessly integrate your **Sony DualSense** (and DualShock 4) into the Windows ecosystem. 

It bridges the gap between Sony hardware and PC gaming by emulating an **Xbox 360 controller**, ensuring 100% compatibility with all games, launchers, and emulators—without the bloat.

![Dx3 Showcase](showcase/demo.gif)

## 🚀 Why Dx3?

Unlike other tools that consume 150MB+ of RAM and require heavy .NET runtimes, Dx3 is built with **Rust** and **Tauri**.

*   **🍃 Ultra-Lightweight:** Occupies **only ~5MB RAM** when minimized to the system tray. The UI engine completely unloads, leaving only a tiny, highly efficient background worker.
*   **⚡ Zero Latency:** Input processing happens in a dedicated native thread, ensuring no input lag.
*   **🔋 Battery Friendly:** Minimal CPU usage means more battery life for your laptop and controller.

## ✨ Key Features

### 🔌 Connectivity & Fixes
*   **Bluetooth "Simple Mode" Fix:** Automatically detects when Windows limits the DualSense capabilities over Bluetooth and switches it to Enhanced Mode. Get **RGB, Rumble, and Triggers wirelessly** without needing DS4Windows.
*   **HidHide Integration:** Built-in support to hide the physical controller from games to prevent the dreaded "Double Input" issue.

### 🎮 Next-Gen Controls
*   **Adaptive Triggers:** Customize the DualSense triggers with modes like:
    *   *Rigid* (Hard stop)
    *   *Section* (Resistance zones)
    *   *Pulse* (Vibration feedback)
*   **Touchpad as Mouse:** Turn the touchpad into a precision trackpad for navigating your desktop from the couch. Includes scroll gestures!

### 🎨 Customization
*   **Visual Remapper:** Beautiful pixel-art interface to remap buttons to Keyboard keys, Mouse clicks, or Xbox actions.
*   **RGB Control:** Full control over the lightbar color and brightness. Includes a battery indicator mode.
*   **Profiles:** Create and switch between configs for different games instantly.

## 📦 Prerequisites

To use Dx3 Controller, you need the following drivers installed:

1.  **[ViGEmBus Driver](https://github.com/nefarius/ViGEmBus/releases/latest)** (Required for Xbox 360 emulation).
2.  **[HidHide](https://github.com/nefarius/HidHide/releases/latest)** (Recommended to hide the original Sony controller).
3.  **[Microsoft Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170)** (Standard Windows runtime).

## 🚀 Installation & Usage

1.  Download the latest release from the [Releases Page](https://github.com/mantukin/dx3/releases).
2.  Install **ViGEmBus** (link above).
3.  Run `dx3.exe`.
4.  Connect your DualSense via USB or Bluetooth.
5.  The app will automatically detect the controller and start the emulation.

> **Pro Tip:** If you encounter connection or stability issues, we highly recommend updating your **DualSense Firmware** to the latest version using the official [Sony Firmware Updater](https://controller.dl.playstation.net/controller/lang/en/2100004.html).

> **Note:** Run the application as **Administrator** if you want to use Keyboard/Mouse mapping in games with anti-cheat protection.

## 🛠️ Building from Source

If you want to modify or build the project yourself:

### Requirements
*   [Node.js](https://nodejs.org/) (v16+)
*   [Rust](https://www.rust-lang.org/) (latest stable)
*   Build Tools for Visual Studio (C++ workload)

### Steps

1.  Clone the repository:
    ```bash
    git clone https://github.com/mantukin/dx3.git
    cd dx3
    ```

2.  Install frontend dependencies:
    ```bash
    npm install
    ```

3.  Run in Development Mode:
    ```bash
    npm run tauri dev
    ```

4.  Build for Release:
    ```bash
    npm run tauri build
    ```
    The output executable will be in `src-tauri/target/release/bundle/msi/`.

## 🤝 Acknowledgments

*   **[ViGEmBus](https://github.com/nefarius/ViGEmBus)** by Nefarius - The magic behind controller emulation.
*   **[Tauri](https://tauri.app/)** - For the amazing lightweight framework.
*   **HidApi** - For communicating with HID devices.

## 📄 License

[MIT License](LICENSE) © 2026 IPD Workshop (mantukin)
