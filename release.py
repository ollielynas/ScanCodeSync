import os
import shutil
import subprocess
from datetime import datetime
from pathlib import Path

import toml

# --- CONFIGURATION ---
RUST_PROJECT_DIR = "ScanCodeSyncDesktop"
WEBSITE_DIR = "HomeWebsite"
CARGO_TOML_PATH = os.path.join(RUST_PROJECT_DIR, "Cargo.toml")
RELEASES_HTML_PATH = os.path.join(WEBSITE_DIR, "releases.html")
WIX_BIN_PATH = os.path.abspath("wix314-binaries")

# Path to makeappx.exe — update this to match your SDK version
MAKEAPPX_PATH = (
    r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.28000.0\x64\makeappx.exe"
)


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


def update_releases_html(version, files):
    date_str = datetime.now().strftime("%B %d, %Y")
    links_html = "".join(
        [
            f'<li><a class="download-btn" href="{version}/{f}">Download {f}</a></li>'
            for f in files
        ]
    )
    content = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Releases | ScanCodeSync</title>
</head>
<body>
    <div class="container">
        <h1>ScanCodeSync Releases</h1>
        <div class="card">
            <span class="version-tag">Latest Release</span>
            <h2>Version {version}</h2>
            <p>Released on {date_str}</p>
            <ul>{links_html}</ul>
        </div>
    </div>
</body>
</html>
"""
    with open(RELEASES_HTML_PATH, "w") as f:
        f.write(content)
    print(f"✅ Generated {RELEASES_HTML_PATH}")


def build_msix(version, exe_path):
    """Packages the built .exe into an MSIX using makeappx.exe"""
    if not os.path.exists(MAKEAPPX_PATH):
        print(f"⚠️  makeappx.exe not found at {MAKEAPPX_PATH} — skipping MSIX build.")
        print("    Install the Windows SDK and update MAKEAPPX_PATH in this script.")
        return None

    print(f"📦 Building MSIX for v{version}...")

    # Staging folder — makeappx packs everything in here
    staging_dir = Path("msix_staging")
    if staging_dir.exists():
        shutil.rmtree(staging_dir)
    staging_dir.mkdir()

    # Copy exe
    shutil.copy2(exe_path, staging_dir / exe_path.name)

    # Copy manifest (update version field to match)
    manifest_src = Path(RUST_PROJECT_DIR) / "AppxManifest.xml"
    manifest_text = manifest_src.read_text()
    # Patch version in manifest — MSIX needs 4-part version (e.g. 1.2.3.0)
    msix_version = version + ".0"
    manifest_text = manifest_text.replace(
        manifest_text[
            manifest_text.find('Version="') + 9 : manifest_text.find(
                '"', manifest_text.find('Version="') + 9
            )
        ],
        msix_version,
    )
    (staging_dir / "AppxManifest.xml").write_text(manifest_text)

    # Copy assets folder
    assets_src = Path(RUST_PROJECT_DIR) / "Assets"
    if assets_src.exists():
        shutil.copytree(assets_src, staging_dir / "Assets")
    else:
        print(
            "⚠️  No Assets folder found — MSIX may fail Store validation without icons."
        )

    # Run makeappx
    output_msix = Path("msix_staging") / f"ScanCodeSync_{version}.msix"
    try:
        subprocess.run(
            [
                MAKEAPPX_PATH,
                "pack",
                "/d",
                str(staging_dir),
                "/p",
                str(output_msix),
                "/nv",
            ],
            check=True,
        )
        print(f"✅ MSIX created: {output_msix}")
        return output_msix
    except subprocess.CalledProcessError:
        print("❌ MSIX build failed.")
        return None


def main():
    # 1. Path Verification
    if not os.path.exists(WIX_BIN_PATH):
        print(f"❌ Error: {WIX_BIN_PATH} folder missing.")
        return

    # 2. Versioning
    data = toml.load(CARGO_TOML_PATH)
    current_version = data["package"]["version"]
    new_version = increment_version(current_version)
    if new_version != current_version:
        data["package"]["version"] = new_version
        with open(CARGO_TOML_PATH, "w") as f:
            toml.dump(data, f)
        print(f"✅ Updated Cargo.toml to v{new_version}")

    # 3. Build MSI
    print(f"🚀 Compiling MSI for v{new_version}...")
    try:
        subprocess.run(
            ["cargo", "wix", "--bin-path", WIX_BIN_PATH],
            cwd=RUST_PROJECT_DIR,
            check=True,
        )
    except subprocess.CalledProcessError:
        print("❌ Build failed. Check the output above.")
        return

    # 4. Build MSIX
    exe_path = Path(RUST_PROJECT_DIR) / "target" / "release" / "ScanCodeSyncDesktop.exe"
    msix_file = None
    if exe_path.exists():
        msix_file = build_msix(new_version, exe_path)
    else:
        print(f"⚠️  Could not find .exe at {exe_path} — skipping MSIX.")

    # 5. Move files to website folder
    dest_path = Path(WEBSITE_DIR) / new_version
    dest_path.mkdir(parents=True, exist_ok=True)

    wix_path = Path(RUST_PROJECT_DIR) / "target" / "wix"
    moved_files = []

    for file in wix_path.glob("*.msi"):
        shutil.copy2(file, dest_path / file.name)
        moved_files.append(file.name)
        print(f"  -> Moved: {file.name}")

    if msix_file and msix_file.exists():
        shutil.copy2(msix_file, dest_path / msix_file.name)
        moved_files.append(msix_file.name)
        print(f"  -> Moved: {msix_file.name}")
        print(f"\n💡 Upload the .msix to Partner Center: https://partner.microsoft.com")

    if moved_files:
        update_releases_html(new_version, moved_files)
        print(f"\n🎉 Success! Files are in {dest_path}")
        os.startfile(os.path.abspath(RELEASES_HTML_PATH))
    else:
        print("⚠️ No files were found to move.")


if __name__ == "__main__":
    main()
