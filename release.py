import os
import shutil
import subprocess
from pathlib import Path

import toml

# --- CONFIGURATION ---
RUST_PROJECT_DIR = "ScanCodeSyncDesktop"
WEBSITE_DIR = "HomeWebsite"
CARGO_TOML_PATH = os.path.join(RUST_PROJECT_DIR, "Cargo.toml")

# Get the absolute path to your local binaries folder
# This ensures it works even when the script changes directories to build
WIX_BIN_PATH = os.path.abspath("wix314-binaries")


def increment_version(version_str):
    major, minor, patch = map(int, version_str.split("."))
    print(f"\n--- ScanCodeSync Release Manager ---")
    print(f"Current version: {version_str}")
    print(f" [1] Patch ({major}.{minor}.{patch + 1})")
    print(f" [2] Minor ({major}.{minor + 1}.0)")
    print(f" [3] Major ({major + 1}.0.0)")
    print(f" [Any other key] Skip increment")

    choice = input("Selection: ")
    if choice == "1":
        return f"{major}.{minor}.{patch + 1}"
    if choice == "2":
        return f"{major}.{minor + 1}.0"
    if choice == "3":
        return f"{major + 1}.0.0"
    return version_str


def main():
    # 1. Verify Binaries exist
    candle_path = os.path.join(WIX_BIN_PATH, "candle.exe")
    if not os.path.exists(candle_path):
        print(f"❌ Error: Could not find candle.exe at {WIX_BIN_PATH}")
        print("Ensure you extracted the zip into the 'wix314-binaries' folder.")
        return

    # 2. Versioning
    if not os.path.exists(CARGO_TOML_PATH):
        print(f"❌ Error: {CARGO_TOML_PATH} not found.")
        return

    data = toml.load(CARGO_TOML_PATH)
    current_version = data["package"]["version"]
    new_version = increment_version(current_version)

    if new_version != current_version:
        data["package"]["version"] = new_version
        with open(CARGO_TOML_PATH, "w") as f:
            toml.dump(data, f)
        print(f"✅ Updated Cargo.toml to v{new_version}")

    # 3. Build the MSI
    print(f"🚀 Building MSI installer for v{new_version}...")
    try:
        # We tell cargo-wix exactly where to find your downloaded binaries
        subprocess.run(
            ["cargo", "wix", "--bin-path", WIX_BIN_PATH],
            cwd=RUST_PROJECT_DIR,
            check=True,
        )
    except subprocess.CalledProcessError:
        print("❌ Build failed. Check the errors above.")
        return

    # 4. Move Files to Website Folder
    dest_path = Path(WEBSITE_DIR) / new_version
    dest_path.mkdir(parents=True, exist_ok=True)

    wix_output_path = Path(RUST_PROJECT_DIR) / "target" / "wix"

    files_moved = 0
    for file in wix_output_path.glob("*.msi"):
        shutil.copy2(file, dest_path / file.name)
        print(f"  -> Moved {file.name} to {dest_path}")
        files_moved += 1

    if files_moved > 0:
        print(f"\n🎉 Success! Release v{new_version} is ready in {WEBSITE_DIR}.")
        # Optional: Open the destination folder in File Explorer
        os.startfile(dest_path)
    else:
        print("\n⚠️ Build finished but no .msi was found. Did you run 'cargo wix init'?")


if __name__ == "__main__":
    main()
