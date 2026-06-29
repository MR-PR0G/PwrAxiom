#!/bin/bash

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}=== Power Axiom Installer ===${NC}\n"

if [ "$EUID" -ne 0 ]; then
  echo -e "${RED}[!] Please run as root (sudo).${NC}"
  exit 1
fi

echo -e "${BLUE}[*] Step 1: Detecting Operating System...${NC}"
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
else
    OS=$(uname -s)
fi
echo -e "${GREEN}[+] OS detected: $OS${NC}"

DEPS_TO_INSTALL=""

echo -e "\n${BLUE}[*] Step 2: Checking Dependencies...${NC}"
case "$OS" in
    ubuntu|debian)
        DEPS="libgtk-4-1 pciutils cpufrequtils linux-tools-generic"
        for pkg in $DEPS; do
            if ! dpkg -s $pkg >/dev/null 2>&1; then
                DEPS_TO_INSTALL="$DEPS_TO_INSTALL $pkg"
            fi
        done
        INSTALL_CMD="apt-get update && apt-get install -y"
        ;;
    arch|manjaro)
        DEPS="gtk4 pciutils cpupower"
        for pkg in $DEPS; do
            if ! pacman -Q $pkg >/dev/null 2>&1; then
                DEPS_TO_INSTALL="$DEPS_TO_INSTALL $pkg"
            fi
        done
        INSTALL_CMD="pacman -Sy --noconfirm"
        ;;
    fedora)
        DEPS="gtk4 pciutils kernel-tools"
        for pkg in $DEPS; do
            if ! rpm -q $pkg >/dev/null 2>&1; then
                DEPS_TO_INSTALL="$DEPS_TO_INSTALL $pkg"
            fi
        done
        INSTALL_CMD="dnf install -y"
        ;;
    *)
        echo -e "${YELLOW}[!] Unknown distribution. Skipping dependency check.${NC}"
        ;;
esac

if [ -n "$DEPS_TO_INSTALL" ]; then
    echo -e "${YELLOW}[!] Missing dependencies detected:${NC} $DEPS_TO_INSTALL"
    read -p "Do you want to install them now? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}[*] Installing dependencies...${NC}"
        eval "$INSTALL_CMD $DEPS_TO_INSTALL"
        echo -e "${GREEN}[+] Dependencies installed successfully.${NC}"
    else
        echo -e "${RED}[!] Cannot proceed without dependencies. Exiting.${NC}"
        exit 1
    fi
else
    if [ "$OS" = "ubuntu" ] || [ "$OS" = "debian" ] || [ "$OS" = "arch" ] || [ "$OS" = "manjaro" ] || [ "$OS" = "fedora" ]; then
        echo -e "${GREEN}[+] All dependencies are already installed.${NC}"
    fi
fi

echo -e "\n${BLUE}[*] Step 3: Installing Core Files...${NC}"
if [ ! -f "pwraxiom" ]; then
    echo -e "${RED}[!] Error: 'pwraxiom' executable not found in current directory.${NC}"
    exit 1
fi

cp pwraxiom /usr/local/bin/
chmod +x /usr/local/bin/pwraxiom
echo -e "${GREEN}[+] Executable installed to /usr/local/bin/pwraxiom${NC}"

ICON_FILE=$(find . -maxdepth 1 -name "pwraxiomicon.*" | head -n 1)
ICON_NAME="system-run"
if [ -n "$ICON_FILE" ]; then
    EXT="${ICON_FILE##*.}"
    cp "$ICON_FILE" "/usr/share/pixmaps/pwraxiom.$EXT"
    # تغییر: استفاده از مسیر کامل برای جلوگیری از گم شدن آیکون در دسکتاپ
    ICON_NAME="/usr/share/pixmaps/pwraxiom.$EXT"
    echo -e "${GREEN}[+] Icon installed to /usr/share/pixmaps/pwraxiom.$EXT${NC}"
else
    echo -e "${YELLOW}[!] Icon not found. Using default system icon.${NC}"
fi

echo -e "\n${BLUE}[*] Step 4: Creating Desktop Entry...${NC}"
cat <<EOF > /usr/share/applications/pwraxiom.desktop
[Desktop Entry]
Type=Application
Name=Power Axiom
Comment=Performance and Power Management Utility
Exec=/usr/local/bin/pwraxiom
Icon=$ICON_NAME
Terminal=false
Categories=System;Settings;
StartupWMClass=pwraxiom
EOF
echo -e "${GREEN}[+] Desktop entry created at /usr/share/applications/pwraxiom.desktop${NC}"

echo -e "\n${BLUE}[*] Step 5: Updating Desktop Database...${NC}"
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications
    echo -e "${GREEN}[+] Desktop database refreshed.${NC}"
else
    echo -e "${YELLOW}[!] 'update-desktop-database' not found. Please log out and log back in to see the icon.${NC}"
fi

echo -e "\n${GREEN}[+] Installation Completed Successfully! You can now run 'pwraxiom'.${NC}"