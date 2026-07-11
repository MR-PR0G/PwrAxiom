#!/usr/bin/env bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0;3m'
BOLD='\033[1m'

echo -e "${BLUE}${BOLD}=========================================${NC}"
echo -e "${BLUE}${BOLD}       Power Axiom Installer v0.2.0      ${NC}"
echo -e "${BLUE}${BOLD}=========================================${NC}"

if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: Please run the installer as root (sudo ./installer.sh).${NC}"
    exit 1
fi

BINARY_SRC="./target/release/power_axiom"
if [ ! -f "$BINARY_SRC" ]; then
    echo -e "${RED}Error: Release binary not found at $BINARY_SRC${NC}"
    echo -e "${YELLOW}Please run 'cargo build --release' first.${NC}"
    exit 1
fi

echo -e "${BLUE}Detecting Linux distribution...${NC}"
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS_ID=$ID
    OS_LIKE=$ID_LIKE
else
    echo -e "${RED}Error: Could not detect OS version.${NC}"
    exit 1
fi

install_deps() {
    local pkgs_arch=("gtk4" "glib2" "libdrm" "egl-wayland" "mesa" "cairo" "pango" "graphene" "dbus" "pciutils" "cpupower" "polkit")
    local pkgs_debian=("libgtk-4-dev" "libglib2.0-dev" "libdrm-dev" "libvulkan1" "libegl-wayland1" "pciutils" "linux-cpupower" "policykit-1")
    local pkgs_fedora=("gtk4-devel" "glib2-devel" "libdrm-devel" "vulkan-loader" "pciutils" "kernel-tools" "polkit")

    case "$OS_ID" in
        arch|manjaro|endeavouros)
            echo -e "${GREEN}Detected Arch-based system.${NC}"
            pacman -S --needed --noconfirm "${pkgs_arch[@]}"
            ;;
        ubuntu|debian|pop|mint)
            echo -e "${GREEN}Detected Debian/Ubuntu-based system.${NC}"
            apt-get update
            apt-get install -y "${pkgs_debian[@]}"
            ;;
        fedora|rhel|centos)
            echo -e "${GREEN}Detected Fedora/RHEL-based system.${NC}"
            dnf install -y "${pkgs_fedora[@]}"
            ;;
        *)
            if [[ "$OS_LIKE" == *"arch"* ]]; then
                pacman -S --needed --noconfirm "${pkgs_arch[@]}"
            elif [[ "$OS_LIKE" == *"debian"* || "$OS_LIKE" == *"ubuntu"* ]]; then
                apt-get update
                apt-get install -y "${pkgs_debian[@]}"
            elif [[ "$OS_LIKE" == *"fedora"* || "$OS_LIKE" == *"rhel"* ]]; then
                dnf install -y "${pkgs_fedora[@]}"
            else
                echo -e "${YELLOW}Warning: Unsupported distribution. Skipping core dependency installation.${NC}"
            fi
            ;;
    esac
}

echo -e "${BLUE}Installing core system dependencies...${NC}"
install_deps

echo -e ""
echo -e "${YELLOW}${BOLD}Optional GPU Monitoring Dependency${NC}"
read -p "Do you want to install NVIDIA driver dependencies for GPU monitoring? (y/N): " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${BLUE}Installing NVIDIA components...${NC}"
    case "$OS_ID" in
        arch|manjaro|endeavouros) pacman -S --needed --noconfirm nvidia nvidia-utils ;;
        ubuntu|debian|pop|mint) apt-get install -y libnvidia-compute-bg ;;
        fedora|rhel|centos) dnf install -y xorg-x11-drv-nvidia-cuda ;;
    esac
else
    echo -e "${YELLOW}Skipped NVIDIA component installation.${NC}"
fi

echo -e "${BLUE}Copying executable to /usr/local/bin...${NC}"
cp "$BINARY_SRC" /usr/local/bin/pwraxiom-core
chmod +x /usr/local/bin/pwraxiom-core

echo -e "${BLUE}Creating graphical privilege wrapper...${NC}"
cat << 'EOF' > /usr/local/bin/pwraxiom
#!/usr/bin/env bash

APP_EXEC="/usr/local/bin/pwraxiom-core"

if command -v pkexec &> /dev/null; then
    pkexec env DISPLAY="$DISPLAY" WAYLAND_DISPLAY="$WAYLAND_DISPLAY" XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" DBUS_SESSION_BUS_ADDRESS="$DBUS_SESSION_BUS_ADDRESS" "$APP_EXEC"
else
    if command -v xhost &> /dev/null; then
        xhost +si:localuser:root > /dev/null 2>&1
        sudo -E dbus-run-session "$APP_EXEC"
    else
        sudo "$APP_EXEC"
    fi
fi
EOF

chmod +x /usr/local/bin/pwraxiom

ICON_SRC="./pwraxiomicon.png"
ICON_DEST="/usr/share/icons/hicolor/256x256/apps/pwraxiom.png"
if [ -f "$ICON_SRC" ]; then
    echo -e "${BLUE}Installing application icon...${NC}"
    mkdir -p /usr/share/icons/hicolor/256x256/apps/
    cp "$ICON_SRC" "$ICON_DEST"
else
    echo -e "${YELLOW}Warning: Icon file pwraxiomicon.png not found. Skipping icon copy.${NC}"
fi

echo -e "${BLUE}Creating Desktop Entry shortcut...${NC}"
DESKTOP_ENTRY="/usr/share/applications/pwraxiom.desktop"
cat << EOF > "$DESKTOP_ENTRY"
[Desktop Entry]
Version=1.0
Type=Application
Name=Power Axiom
Comment=Hardware Performance and Power Management Utility
Exec=/usr/local/bin/pwraxiom
Icon=pwraxiom
Terminal=false
Categories=System;Monitor;GTK;
StartupNotify=true
EOF

chmod +x "$DESKTOP_ENTRY"

if command -v update-desktop-database &> /dev/null; then
    update-desktop-database /usr/share/applications
fi

echo -e ""
echo -e "${GREEN}${BOLD}=========================================${NC}"
echo -e "${GREEN}${BOLD}   Power Axiom Installed Successfully!   ${NC}"
echo -e "${GREEN}${BOLD}=========================================${NC}"
echo -e "${YELLOW}You can now run the app via terminal using: ${BOLD}pwraxiom${NC}"
echo -e "${YELLOW}Or find it in your desktop applications menu.${NC}"