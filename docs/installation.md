# Installation

Download and install Kindling on your platform.

## Download

Get the latest release from the [GitHub Releases page](https://github.com/smith-and-web/kindling/releases).

### Available Downloads

| Platform              | File                                | Description                                   |
| --------------------- | ----------------------------------- | --------------------------------------------- |
| macOS                 | `Kindling_<version>_universal.dmg`  | One build for both Apple Silicon and Intel    |
| Windows               | `Kindling_<version>_x64-setup.exe`  | Windows installer (recommended)               |
| Windows               | `Kindling_<version>_x64_en-US.msi`  | MSI package, for managed or scripted installs |
| Linux                 | `Kindling_<version>_amd64.AppImage` | Portable, works on most distros               |
| Linux (Debian/Ubuntu) | `Kindling_<version>_amd64.deb`      | Native package for apt-based systems          |
| Linux (Fedora/RHEL)   | `Kindling-<version>-1.x86_64.rpm`   | Native package for dnf/rpm-based systems      |

The `.sig` files next to each download are update signatures used by the in-app updater; you don't need them for a manual install.

---

## Verify Your Download

To ensure you downloaded the authentic Kindling release:

1. **Download only from official sources**: [GitHub Releases](https://github.com/smith-and-web/kindling/releases)
2. **Verify checksums (optional but recommended)**:
   - Download `checksums.sha256` from the same release
   - Verify the file with your platform:

```bash
# macOS / Linux
shasum -a 256 -c checksums.sha256 --ignore-missing

# Windows (PowerShell)
certutil -hashfile Kindling_<version>_x64_en-US.msi SHA256
```

If your checksum doesn't match, delete the file and re-download from the official release page.

## Trust Signals

- Kindling is open source and auditable: [github.com/smith-and-web/kindling](https://github.com/smith-and-web/kindling)
- Release builds are produced by GitHub Actions in this repository
- Release assets include checksums for download verification

---

## macOS Installation

Kindling is code-signed and notarized by Apple. It should open without Gatekeeper warnings.

1. Double-click the `.dmg` file to mount it
2. Drag **Kindling** to your **Applications** folder
3. Double-click **Kindling** in your Applications folder to launch it

---

## Windows Installation

Kindling is not signed with a Windows code signing certificate. Windows SmartScreen may show a warning when you first run the installer.

### Bypassing SmartScreen

1. Download the `_x64-setup.exe` installer (or the `.msi` package)
2. Double-click to run it
3. If you see **"Windows protected your PC"**:
   - Click **More info**
   - Click **Run anyway**
4. Follow the installation wizard
5. Launch Kindling from the Start menu

After installation, Kindling will run without warnings.

### Why This Warning Appears

SmartScreen warns about apps that don't have an established reputation with Microsoft. As more users download and run Kindling, this warning will eventually disappear. Code signing certificates are expensive ($200-500/year), so we've opted to ship unsigned for now.

---

## Linux Installation

### AppImage (Recommended)

AppImage is a portable format that works on most Linux distributions without installation.

```bash
# Download the AppImage
# Make it executable
chmod +x Kindling_*.AppImage

# Run it
./Kindling_*.AppImage
```

#### Optional: Desktop Integration

To add Kindling to your application menu:

```bash
# Install AppImageLauncher (recommended)
# Or manually create a .desktop file

# Move AppImage to a permanent location
mkdir -p ~/Applications
mv Kindling_*.AppImage ~/Applications/

# Create desktop entry
cat > ~/.local/share/applications/kindling.desktop << EOF
[Desktop Entry]
Name=Kindling
Exec=$HOME/Applications/Kindling_*.AppImage
Icon=kindling
Type=Application
Categories=Office;Writing;
EOF
```

### Debian/Ubuntu (.deb)

For Debian-based distributions (Ubuntu, Linux Mint, Pop!\_OS, etc.):

```bash
# Install the package
sudo dpkg -i Kindling_*_amd64.deb

# If there are dependency issues
sudo apt-get install -f
```

Launch Kindling from your application menu or run `kindling` in the terminal.

#### Uninstalling

```bash
sudo apt remove kindling
```

### Fedora/RHEL (.rpm)

For RPM-based distributions (Fedora, RHEL, openSUSE, etc.):

```bash
# Install the package
sudo dnf install ./Kindling-*-1.x86_64.rpm
```

Launch Kindling from your application menu or run `kindling` in the terminal.

---

## Troubleshooting

### macOS: App crashes immediately

The macOS download is a single universal build (`Kindling_<version>_universal.dmg`) that runs natively on both Apple Silicon and Intel Macs, so there is no architecture to choose. Delete the app, re-download the DMG from the [Releases page](https://github.com/smith-and-web/kindling/releases) and drag it to Applications again.

### Windows: Installer fails silently

Try running the installer as Administrator:

1. Right-click the `_x64-setup.exe` or `.msi` file
2. Select **Run as administrator**

### Linux: AppImage won't start

Ensure FUSE is installed:

```bash
# Ubuntu/Debian
sudo apt install libfuse2

# Fedora
sudo dnf install fuse
```

### Linux: No application icon

AppImages don't always integrate with desktop environments automatically. Use AppImageLauncher or create a `.desktop` file manually (see above).

---

## Building from Source

If you prefer to build Kindling yourself, see the [README](https://github.com/smith-and-web/kindling#from-source) for instructions.
