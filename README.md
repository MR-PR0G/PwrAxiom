# ⚡ Power Axiom

![Version](https://img.shields.io/badge/Version-0.2.0-blue.svg)
![Platform](https://img.shields.io/badge/Platform-Linux-lightgrey.svg)
![GTK](https://img.shields.io/badge/GUI-GTK4-green.svg)
![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)

**Power Axiom** is an advanced, system-level hardware performance and power management utility for Linux. Completely rewritten in **Rust** and powered by a fluid, glassmorphism-inspired **GTK4** interface, it grants users absolute control over their CPU, GPU, and PCIe states.

Whether you need to squeeze every drop of battery life out of your laptop or unleash maximum performance for heavy developer workloads and gaming, Power Axiom configures your Linux kernel parameters safely, instantly, and efficiently.

---

<p align="center">
  <table border="0" style="border-collapse: collapse; border-style: none;" align="center">
    <tr>
      <td align="center" width="50%" style="border: none; padding: 5px;">
        <img src="/Demos/1.png" alt="Power Axiom Dashboard" style="width: 100%; max-width: 100%; border-radius: 8px;">
      </td>
      <td align="center" width="50%" style="border: none; padding: 5px;">
        <img src="/Demos/2.png" alt="Performance Mode" style="width: 100%; max-width: 100%; border-radius: 8px;">
      </td>
    </tr>
    <tr>
      <td align="center" width="50%" style="border: none; padding: 5px;">
        <img src="/Demos/3.png" alt="Dynamic Theming" style="width: 100%; max-width: 100%; border-radius: 8px;">
      </td>
      <td align="center" width="50%" style="border: none; padding: 5px;">
        <img src="/Demos/4.png" alt="Save Mode" style="width: 100%; max-width: 100%; border-radius: 8px;">
      </td>
    </tr>
  </table>
</p>

---

## ✨ Key Features

* 🎛️ **Hardware Profiles:** Seamlessly switch between high-performance execution, adaptive balance, and aggressive power-saving structures.
* 📊 **Real-time Monitoring:** Internal **gatherer** engine tracking per-core CPU frequencies, live Package wattage, and multi-vendor GPU metrics (NVIDIA/AMD/Intel) through safe static `nvtop` linkers.
* 🧠 **Deep Kernel Integration:** Direct communication with Linux power governors, Turbo Boost toggles, and sysfs kernel power parameters.
* 🎮 **GPU Power States:** Auto-manages discrete GPU runtime states or forces power-down hooks for extreme hardware power preservation.
* 🎨 **Liquid Glass Aesthetics:** Smooth CSS interaction overlays, dynamic responsive spotlight border animations with boundary clamps, and modern dark-mode layouts.
* 🛡️ **Polkit Secure Execution:** Safe, sandboxed system-level modifications with live desktop signal integration via D-Bus (`dbus-crossroads`).
* 🐧 **Cross-Distro Installer:** Automated environment detection and safe binary caching across popular distributions.

---

## 🚀 Power Profiles & Customization

| Profile | Description |
| :--- | :--- |
| ⚡ **Performance** | Applies the kernel `performance` governor, activates Turbo Boost / P-States scaling limits, and allows maximum hardware frequency response. |
| ⚖️ **Balanced** | Restores the default `schedutil` governor, allowing smooth frequency scaling based on immediate CPU/GPU rendering demands. |
| 🔋 **Save** | Standard power saving. Applies the `powersave` governor, deactivates Turbo Boost, and optimizes internal links to extend battery runtime. |
| 🛠️ **Custom Setup** | **[COMING SOON]** Take ultimate control. Design your own hardware profiles by locking down specific clock ranges, undervolting thresholds, custom governor bindings, and dedicated fan curves per application. |

---

## 🛠️ Installation 
1. Go to the [Releases](https://github.com/MR-PR0G/pwraxiom/releases) page and download the latest.
2. Extract the archive and run the installer:
```bash
tar -xvf power_axiom-v0.2.0-x86_64.tar.gz
cd power_axiom-v0.2.0
chmod +x installer.sh
sudo ./installer.sh
```
The installer will automatically:
- Detect your Linux distribution (Arch Linux, Debian/Ubuntu, Fedora/RHEL).
- Install missing runtime dependencies (gtk4, glib2, libdrm, pciutils, cpupower).
- Provide an interactive prompt to pull hardware-specific components (e.g., proprietary NVIDIA compute modules).
- Verify and link the tracked standalone production binary directly to your global environment at /usr/local/bin/pwraxiom.
- Register the custom application vector arts (pwraxiomicon.png) and deploy the official system-wide .desktop workspace shortcut.
## 💻 Usage
Once installed globally, you can launch the application directly from your desktop environment's application menu (search for Power Axiom), or invoke it via your terminal interface:
```
pwraxiom
```
⚠️ Note: Administrative privileges via a Polkit popup window will only be requested dynamically by the system daemon when you actively switch or apply a new hardware power profile.
## 📂 Project Structure 
```
pwraxiom/
├── Cargo.toml          # Workspace root configuration
├── Cargo.lock          # Dependency lockfile
├── installer.sh        # Smart automatic cross-distro setup tool
├── pwraxiomicon.png    # High-resolution glassmorphism icon
├── README.md           # Documentation
├── Demos/              # Visual screenshots & previews for README
├── src/                # Front-end UI & Application Logic
│   ├── main.rs         # GTK4 window composition & entry point
│   ├── backend/        # UI-to-daemon bridge & Polkit IPC handlers
│   └── ui/             # Glassmorphism CSS, spotlight animations & state listeners
└── gatherer/           # Core Hardware Engine (Backend Crate)
    ├── Cargo.toml      # Subsystem library definition
    ├── build/          # Native C compilation hooks (nvtop integration)
    ├── 3rdparty/       # External native bindings & submodules
    └── src/            # Kernel sysfs abstractions, CPU/GPU metric parsers
```
## ⚙️ Hacking & Compiling From Source
## ⚙️ Hacking & Compiling From Source

**For developers, power-users, or package maintainers who prefer local optimizations, strict architecture tailoring, or system auditing, Power Axiom provides a pure Rust workspace. You can rebuild the entire binary target natively in a single stage.**

### 1. Developer Dependencies
Before building from source, you must install the required native development libraries, C compilers, and graphic subsystem bindings on your host device:

| Distribution | Command to Install Build Dependencies |
| :--- | :--- |
| **Arch Linux** | `sudo pacman -S --needed base-devel rust cargo gtk4 glib2 libdrm pciutils cpupower` |
| **Debian / Ubuntu** | `sudo apt install -y build-essential rustc cargo libgtk-4-dev libglib2.0-dev libdrm-dev pciutils linux-cpupower` |
| **Fedora / RHEL** | `sudo dnf groupinstall -y "Development Tools" && sudo dnf install -y rust cargo gtk4-devel glib2-devel libdrm-devel pciutils kernel-tools` |

### 2. Native Monolithic Compilation
**Execute the commands below to purge any existing local build caches and force a high-optimization release profile utilizing Fat LTO (Link-Time Optimization) and single codegen units for maximum code stripping, hardware-specific acceleration, and execution speed:**

```bash
# Flush target work-trees and artifact caches
cargo clean

# Boost allocation limits to prevent compiler stack overflow during deep LLVM optimization passes
export RUST_MIN_STACK=134217728

# Compile optimized standalone production binary with statically linked low-level C layers
cargo build --release --all-features
```
### 💡 Pro-Tip (Target Architecture Tailoring):
To generate an executable highly optimized for your machine's exact CPU architecture (enabling special instruction sets like AVX-512, FMA, etc.), pass the host target flag to the Rust compiler before building:
```
RUSTFLAGS="-C target-cpu=native" cargo build --release --all-features
```
### 3. Immediate Binary Verification & Testing
Once completed, the independent native executable will be generated at `target/release/power_axiom`. You can bypass global deployment and test your modifications instantly or manually run the installer to lock down your new build:
```
# Execute local target directly with isolated environment testing
./target/release/power_axiom
# Or register your custom local build into system-wide binaries via the tracking script
sudo ./installer.sh
```

## 📜 Acknowledgments


This project stands on the shoulders of giants. Special thanks to the open-source community and the following upstream project:


* **[Mission Center]([https://github.com/mishaaq/mission-center](https://github.com/Slimbook-Team/mission-center):** The core hardware metrics parsing logic in our `gatherer` subsystem is adapted from Mission Center's process and usage gathering tools under the GPL license. 
