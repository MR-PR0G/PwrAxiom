# ⚡ Power Axiom

![Version](https://img.shields.io/badge/Version-0.1.1-blue.svg)
![Platform](https://img.shields.io/badge/Platform-Linux-lightgrey.svg)
![GTK](https://img.shields.io/badge/GUI-GTK4-green.svg)
![C](https://img.shields.io/badge/Language-C-blue.svg)

**Power Axiom** is an advanced, system-level power and performance management utility for Linux. Written in C and powered by GTK4, it grants users absolute control over their CPU, GPU, and PCIe states through a sleek, hardware-accelerated interface.

Whether you need to squeeze every drop of battery life out of your laptop or unleash maximum overclocked performance for gaming and heavy workloads, Power Axiom configures your Linux kernel parameters safely and instantly.
---

## ✨ Key Features

* 🎛️ **5 Hardware Profiles:** Instantly switch between Ultra Performance, Performance, Balanced, Save, and Ultra Save.
* 📊 **Real-time Monitoring:** Built-in dashboard tracking per-core CPU frequencies, GPU status (Active/Disabled/Clock speeds), and Live Package/Battery Wattage.
* 🧠 **Deep Kernel Integration:** Controls CPU Governors, Turbo Boost, Intel P-States, AMD P-States, and PCIe ASPM (Active State Power Management).
* 🎮 **GPU Management:** Forces dGPU into `D3Cold` (Deep Sleep) for extreme battery savings, or maxes out clocks for heavy rendering.
* 🎨 **Dynamic Theming:** 10 built-in color schemes with glowing, modern GTK4 CSS styling.
* 🛡️ **Secure Execution:** Utilizes `pkexec` (Polkit) for secure, sandboxed root-level hardware modifications with live progress tracking.
* 🐧 **Cross-Distro Support:** Automated dependency resolution for Arch Linux, Ubuntu/Debian, and Fedora.

---

## 🚀 Power Modes Explained

| Mode | Description |
| :--- | :--- |
| 🔥 **Ultra Performance** | Overclocks & locks minimum frequencies (>1.5GHz), maxes out GPU performance states, and enforces maximum PCIe power. |
| ⚡ **Performance** | Applies `performance` governor, enables Turbo Boost, and allows maximum CPU/GPU frequency scaling. |
| ⚖️ **Balanced** | Restores the default `schedutil` governor, auto-manages GPU, and returns hardware to factory default power management. |
| 🔋 **Save** | Standard battery saving. Applies `powersave` governor, disables Turbo Boost, and puts PCIe into powersave mode. |
| 🛑 **Ultra Save** | Extreme battery preservation. Hard-caps CPU cores (Cores 0,1 to 1.5GHz; others to 800MHz), disables dGPU entirely (turns off via vgaswitcheroo), and enforces minimum Link Power. |

---

## 🛠️ Installation

Power Axiom comes with a smart installer that automatically detects your Linux distribution, installs the required dependencies, and sets up the desktop entry.

### 1. Clone the repository
```bash
git clone [https://github.com/MR-PR0G/pwraxiom.git](https://github.com/MR-PR0G/pwraxiom.git)
cd pwraxiom‍
```
2. Run the Installer

Execute the installation script with root privileges:
```bash
sudo ./install.sh
```
The installer will:
  -  Detect your OS (Arch, Debian/Ubuntu, Fedora).

  - Prompt you to install missing dependencies (gtk4, pciutils, cpupower/linux-tools).

  - Copy the executable to /usr/local/bin/pwraxiom.

  - Install the application icon and create a Desktop entry.

💻 Usage

You can launch the application directly from your desktop environment's app menu (search for Power Axiom), or start it via terminal:
```
pwraxiom
```
Note: Administrative privileges (via a Polkit popup) will only be requested when you apply a power mode.
📂 Project Structure
```Plaintext
pwraxiom/
├── pwraxiom            # Compiled Executable
├── pwraxiomicon.png    # Application Icon
├── install.sh          # Smart Installer Script
├── README.md           # Documentation
└── src/                # C Source Code
    ├── main.c          # GUI & App Logic
    ├── monitor.c       # Hardware Monitoring Engine
    ├── monitor.h       # Monitor Header
    ├── power_core.c    # Script Generation & Kernel Configs
    └── power_core.h    # Core Header
```
📝 Compiling from Source

If you wish to modify the code and compile it yourself, ensure you have the gcc and gtk4 development packages installed, then run:
```
cd src
gcc main.c monitor.c power_core.c -o ../pwraxiom $(pkg-config --cflags --libs gtk4) -lm
```
