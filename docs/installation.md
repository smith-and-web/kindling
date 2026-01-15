# Installation

Download and install Kindling on your platform.

## Download

Get the latest release from the [GitHub Releases page](https://github.com/smith-and-web/kindling/releases).

### Available Downloads

| Platform | File | Description |
|----------|------|-------------|
| macOS (Apple Silicon) | `Kindling_*_aarch64.dmg` | For M1, M2, M3 Macs |
| macOS (Intel) | `Kindling_*_x64.dmg` | For Intel-based Macs |
| Windows | `Kindling_*_x64-setup.msi` | Windows installer |
| Linux | `Kindling_*_amd64.AppImage` | Portable, works on most distros |
| Linux (Debian/Ubuntu) | `kindling_*_amd64.deb` | Native package for apt-based systems |

---

## macOS Installation

Kindling is not code-signed with an Apple Developer certificate. When you first open the app, macOS will show a security warning.

### macOS Sequoia (15.0) and Later

1. Double-click the `.dmg` file to mount it
2. Drag **Kindling** to your **Applications** folder
3. Try to open Kindling — you'll see a message saying it "cannot be opened because the developer cannot be verified"
4. Open **System Settings** → **Privacy & Security**
5. Scroll down to the **Security** section
6. You'll see a message about Kindling being blocked — click **Open Anyway**
7. Enter your password if prompted
8. Click **Open** in the confirmation dialog

After this one-time setup, Kindling will open normally.

### macOS Sonoma (14.0) and Earlier

1. Double-click the `.dmg` file to mount it
2. Drag **Kindling** to your **Applications** folder
3. **Right-click** (or Control-click) on Kindling in your Applications folder
4. Select **Open** from the context menu
5. Click **Open** in the dialog that appears

After this one-time setup, Kindling will open normally.

### Alternative: Terminal Method

If the above methods don't work, you can remove the quarantine attribute:

```bash
xattr -cr /Applications/Kindling.app
```

Then open Kindling normally.

---

## Windows Installation

Kindling is not signed with a Windows code signing certificate. Windows SmartScreen may show a warning when you first run the installer.

### Bypassing SmartScreen

1. Download the `.msi` installer
2. Double-click to run the installer
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

For Debian-based distributions (Ubuntu, Linux Mint, Pop!_OS, etc.):

```bash
# Install the package
sudo dpkg -i kindling_*.deb

# If there are dependency issues
sudo apt-get install -f
```

Launch Kindling from your application menu or run `kindling` in the terminal.

#### Uninstalling

```bash
sudo apt remove kindling
```

---

## Troubleshooting

### macOS: "App is damaged and can't be opened"

This usually means the quarantine attribute is corrupted. Remove it with:

```bash
xattr -cr /Applications/Kindling.app
```

### macOS: App crashes immediately

Ensure you downloaded the correct version:
- **Apple Silicon** (M1/M2/M3): `aarch64.dmg`
- **Intel**: `x64.dmg`

Check your Mac's chip: Apple menu → About This Mac → Chip/Processor.

### Windows: Installer fails silently

Try running the installer as Administrator:
1. Right-click the `.msi` file
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
