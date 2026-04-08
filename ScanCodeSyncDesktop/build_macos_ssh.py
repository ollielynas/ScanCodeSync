import os
import subprocess
from turtle import down

# Configuration
IP = "100.83.221.108"
USER = "ollie"
REMOTE_DIR = "~/Projects/ScanCodeSync"
APP_NAME = "ScanCodeSync"
BINARY_NAME = "ScanCodeSyncDesktop"  # Must match 'name' in [package]


def run_build_rust_over_ssh():

    # Get the directory where THIS script (macos.py) lives
    script_dir = os.path.dirname(os.path.abspath(__file__))

    # Change the working directory to the script's folder
    os.chdir(script_dir)

    print("--- Packaging local source ---")
    subprocess.run(
        ["tar", "--exclude=target", "-czf", "project.tar.gz", "."], check=True
    )

    print(f"--- Uploading to {IP} ---")
    subprocess.run(
        ["scp", "project.tar.gz", f"{USER}@{IP}:~/project.tar.gz"], check=True
    )

    remote_commands = f"""
    export PATH="$HOME/.cargo/bin:$PATH"
    cd {REMOTE_DIR}

    echo "--- Step 1: Building Rust Binary ---"
    tar -xzmf ~/project.tar.gz -C {REMOTE_DIR}
    cargo build --release

    if [ ! -f "target/release/{BINARY_NAME}" ]; then
        echo "ERROR: Binary not found!"
        exit 1
    fi

    echo "--- Step 2: Assembling {APP_NAME}.app ---"
    rm -rf "{APP_NAME}.app" ScanCodeSync_macOS.zip
    mkdir -p "{APP_NAME}.app/Contents/MacOS"
    mkdir -p "{APP_NAME}.app/Contents/Resources"

    cp "target/release/{BINARY_NAME}" "{APP_NAME}.app/Contents/MacOS/ScanCodeSync"
    [ -f "icon.icns" ] && cp "icon.icns" "{APP_NAME}.app/Contents/Resources/icon.icns"

    cat <<EOF > "{APP_NAME}.app/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://apple.com">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>ScanCodeSync</string>
    <key>CFBundleIdentifier</key>
    <string>com.yourname.scancodesync</string>
    <key>CFBundleName</key>
    <string>ScanCodeSync</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.0.2</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.12</string>
</dict>
</plist>
EOF

    echo "--- Step 3: Ad-Hoc Signing ---"
    codesign --force --deep --sign - "{APP_NAME}.app"



    echo "--- Creating DMG Installer ---"
    # Create the DMG from the .app folder
    hdiutil create -volname "ScanCodeSync Installer" -srcfolder "{APP_NAME}.app" -ov -format UDZO "{APP_NAME}.dmg"

    """

    print("--- Executing remote build ---")
    subprocess.run(["ssh", f"{USER}@{IP}", remote_commands], check=True)

    # 5. Download the result back to Windows
    print("--- Downloading finished bundle ---")
    download = subprocess.run(
        ["scp", f"{USER}@{IP}:{REMOTE_DIR}/{APP_NAME}.dmg", "."], check=True
    )
    return download.returncode == 0


if __name__ == "__main__":
    try:
        run_build_rust_over_ssh()
    except Exception as e:
        print(f"Process failed: {e}")
    input()
