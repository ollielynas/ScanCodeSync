import ctypes
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

import toml

import ScanCodeSyncDesktop.build_linux as linux
import ScanCodeSyncDesktop.build_macos_ssh as macos


def is_admin() -> bool:
    try:
        return bool(ctypes.windll.shell32.IsUserAnAdmin())
    except Exception:
        return False


def relaunch_as_admin():
    """Relaunch this script elevated via the UAC prompt, then exit this process."""
    script = os.path.abspath(sys.argv[0])
    params = " ".join(f'"{a}"' for a in sys.argv[1:])
    # 'runas' verb triggers the UAC elevation dialog
    ret = ctypes.windll.shell32.ShellExecuteW(
        None,  # hwnd
        "runas",  # verb
        sys.executable,  # program to run elevated
        f'"{script}" {params}'.strip(),
        None,  # working directory (inherit current)
        1,  # SW_SHOWNORMAL
    )
    # ShellExecuteW returns > 32 on success
    if ret <= 32:
        print(
            f"❌ Could not request elevation (error code {ret}). Try running as Administrator manually."
        )
        sys.exit(1)
    sys.exit(0)  # Original process exits; elevated one takes over


# --- CONFIGURATION ---
RUST_PROJECT_DIR = "ScanCodeSyncDesktop"
WEBSITE_DIR = "HomeWebsite"
CARGO_TOML_PATH = os.path.join(RUST_PROJECT_DIR, "Cargo.toml")
RELEASES_HTML_PATH = os.path.join(WEBSITE_DIR, "releases.html")
WIX_BIN_PATH = os.path.abspath("wix314-binaries")

MAKEAPPX_PATH = (
    r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.28000.0\x64\makeappx.exe"
)

# Windows App Certification Kit CLI — ships with the Windows SDK
# Typical location; update if your SDK version differs
WACK_PATH = r"C:\Program Files (x86)\Windows Kits\10\App Certification Kit\appcert.exe"

# Base URL where your website serves the release files.
# The manifest will construct download URLs as: BASE_RELEASE_URL/{version}/{filename}
BASE_RELEASE_URL = "https://sync-home.ollielynas.com"

# Maps file extensions to the platform keys used in latest.json.
# Add extra entries here if you ever ship arm64 Windows or universal macOS builds.
EXTENSION_TO_PLATFORM = {
    ".msi": "windows-x86_64",
    ".dmg": "darwin-x86_64",
    ".flatpak": "linux-x86_64",
}


def sha256_of_file(path: Path) -> str:
    """Return a lowercase hex SHA-256 digest prefixed with 'sha256:'."""
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return f"sha256:{h.hexdigest()}"


def generate_update_manifest(version: str, dest_path: Path, website_dir: str):

    platforms: dict[str, dict] = {}

    for artifact in sorted(dest_path.iterdir()):
        if not artifact.is_file():
            continue
        platform_key = EXTENSION_TO_PLATFORM.get(artifact.suffix.lower())
        if platform_key is None:
            continue  # .xml WACK reports, .msix, etc. — skip

        print(f"   Hashing {artifact.name} …", end=" ", flush=True)
        digest = sha256_of_file(artifact)
        print("done")

        platforms[platform_key] = {
            "url": f"{BASE_RELEASE_URL}/{version}/{artifact.name}",
            "signature": digest,
        }

    if not platforms:
        print("⚠️  No recognised artifacts found — update manifest not written.")
        return

    manifest = {
        "version": version,
        "notes": f"Release v{version}",
        "pub_date": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "platforms": platforms,
    }

    manifest_json = json.dumps(manifest, indent=2)

    # Canonical location the updater fetches (overwrite on every release)
    latest_path = Path(website_dir) / "latest.json"
    latest_path.write_text(manifest_json, encoding="utf-8")
    print(f"✅ Update manifest written → {latest_path}")

    # Versioned copy for auditing / rollback reference
    versioned_path = dest_path / "update-manifest.json"
    versioned_path.write_text(manifest_json, encoding="utf-8")
    print(f"   Versioned copy          → {versioned_path}")


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


def is_version_dir(path: Path) -> bool:
    return path.is_dir() and bool(re.fullmatch(r"\d+\.\d+\.\d+", path.name))


def version_sort_key(version_str: str):
    return tuple(int(x) for x in version_str.split("."))


def scan_releases(website_dir: str) -> list[dict]:

    base = Path(website_dir)
    releases = []

    for entry in base.iterdir():
        if not is_version_dir(entry):
            continue

        # Collect any .msi / .msix files that actually exist in this folder
        files = sorted(
            f.name
            for f in entry.iterdir()
            if f.is_file() and f.suffix.lower() in {".msi", ".msix", ".dmg", ".flatpak"}
        )

        if files:
            releases.append({"version": entry.name, "files": files})

    # Sort newest version first
    releases.sort(key=lambda r: version_sort_key(r["version"]), reverse=True)
    return releases


def render_release_card(release: dict, is_latest: bool) -> str:
    version = release["version"]
    files = release["files"]

    badge = (
        '<span class="version-tag">Latest Release</span>\n            '
        if is_latest
        else ""
    )

    links_html = "".join(
        f'<li><a class="download-btn" href="{version}/{f}">Download {f}</a></li>'
        for f in files
    )

    return f"""
        <div class="card">
            {badge}<h2>Version {version}</h2>
            <ul>{links_html}</ul>
        </div>"""


def update_releases_html(website_dir: str):

    releases = scan_releases(website_dir)

    if not releases:
        print("No release directories found — releases.html not updated.")
        return

    cards_html = "".join(
        render_release_card(r, is_latest=(i == 0)) for i, r in enumerate(releases)
    )

    latest_version = releases[0]["version"]
    date_str = datetime.now().strftime("%B %d, %Y")

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
        <p>Latest version: {latest_version} - page updated {date_str}</p>
        {cards_html}
        <a href="https://github.com/ollielynas/ScanCodeSync">source code</a>
    </div>
</body>
</html>
"""
    releases_path = os.path.join(website_dir, "releases.html")
    with open(releases_path, "w") as f:
        f.write(content)
    print(f"Generated {releases_path} ({len(releases)} version(s) listed)")


def build_msix(version, exe_path):
    if not os.path.exists(MAKEAPPX_PATH):
        print(f"makeappx.exe not found at {MAKEAPPX_PATH} — skipping MSIX build.")
        print("Install the Windows SDK and update MAKEAPPX_PATH in this script.")
        return None

    print(f"Building MSIX for v{version}...")

    staging_dir = Path("msix_staging")
    if staging_dir.exists():
        shutil.rmtree(staging_dir)
    staging_dir.mkdir()

    shutil.copy2(exe_path, staging_dir / exe_path.name)

    manifest_src = Path(RUST_PROJECT_DIR) / "AppxManifest.xml"
    manifest_text = manifest_src.read_text()
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

    assets_src = Path(RUST_PROJECT_DIR) / "Assets"
    if assets_src.exists():
        shutil.copytree(assets_src, staging_dir / "Assets")
    else:
        print("No Assets folder found — MSIX may fail Store validation without icons.")

    output_msix = staging_dir / f"ScanCodeSync_{version}.msix"
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


def build_linux():
    if input("would you like to build for Linux? [y/n]") in ["n", "N"]:
        return False
    return linux.build_for_linux_using_wsl()


def build_macos():
    if input("would you like to build for MacOS? [y/n]") in ["n", "N"]:
        return False
    return macos.run_build_rust_over_ssh()


def build_windows():
    if input("would you like to build for Windows? [y/n]") in ["n", "N"]:
        return False
    return True


def run_wack(msix_path: Path, report_dir: Path) -> Path | None:

    if not os.path.exists(WACK_PATH):
        print(f"⚠️  appcert.exe not found at {WACK_PATH} — skipping WACK.")
        print(
            "    Install the Windows App Certification Kit via the Windows SDK installer."
        )
        return None

    report_path = report_dir / f"WACKReport_{msix_path.stem}"

    print(f"🔍 Running Windows App Cert Kit on {msix_path.name} ...")
    print(f"   Report will be saved to: {report_path}")
    print(
        "   (This may take a few minutes — the app will be launched and tested silently)"
    )

    try:
        result = subprocess.run(
            [
                WACK_PATH,
                "reset",  # clear any previous session
            ],
            check=False,  # reset may return non-zero; ignore
        )

        msix_abs = str(msix_path.resolve())
        report_abs = str(report_path.resolve())

        subprocess.run(
            [
                WACK_PATH,
                "test",
                "-appxpackagepath",
                msix_abs,
                "-reportoutputpath",
                report_abs,
            ],
            check=True,
        )

        if report_path.exists():
            # Sniff the top-level PASS/FAIL out of the XML so we can log it
            report_text = report_path.read_text(encoding="utf-8", errors="replace")
            if 'OVERALL_RESULT="PASS"' in report_text:
                print(f"✅ WACK result: PASS  - {report_path.name}")
            elif (
                'OVERALL_RESULT="PASSED WITH WARNINGS"' in report_text
                or "WARNING" in report_text
            ):
                print(
                    f"⚠️  WACK result: PASSED WITH WARNINGS — review {report_path.name} before submitting to the Store"
                )
            elif 'OVERALL_RESULT="FAIL"' in report_text:
                print(
                    f"❌ WACK result: FAIL  — open the report for details: {report_path}"
                )
            else:
                print(
                    f"⚠️  WACK finished but result could not be determined — check {report_path}"
                )
            return report_path
        else:
            print("⚠️  WACK ran but no report file was produced.")
            return None

    except subprocess.CalledProcessError as exc:
        print(f"❌ WACK exited with code {exc.returncode} — check output above.")
        return None
    except PermissionError:
        print(
            "❌ WACK was denied access. The UAC elevation may have been cancelled or failed."
        )
        return None


def create_github_release(version: str, files: list[str], dest_path: Path):
    tag = f"v{version}"
    title = f"ScanCodeSync {tag}"

    # Check gh is available
    if shutil.which("gh") is None:
        print("⚠️  GitHub CLI (gh) not found — skipping GitHub release.")
        print("   Install it from https://cli.github.com/")
        return

    print(f"\n🐙 Creating GitHub release {tag}...")

    file_paths = [str(dest_path / f) for f in files if (dest_path / f).exists()]

    if not file_paths:
        print("⚠️  No files found to attach to the GitHub release.")
        return

    try:
        subprocess.run(
            [
                "gh",
                "release",
                "create",
                tag,
                "--title",
                title,
                "--notes",
                f"Release {tag}",
                "--latest",
                *file_paths,
            ],
            check=True,
        )
        print(
            f"✅ GitHub release created: https://github.com/ollielynas/ScanCodeSync/releases/tag/{tag}"
        )
    except subprocess.CalledProcessError:
        print("❌ GitHub release failed. Run 'gh auth login' if not authenticated.")


def main():
    # 1. Elevation check — only needed when WACK is available
    if os.path.exists(WACK_PATH) and not is_admin():
        print("⚠️  The Windows App Cert Kit requires Administrator privileges.")
        answer = input("   Request elevation via UAC now? [Y/n]: ").strip().lower()
        if answer in ("", "y", "yes"):
            relaunch_as_admin()  # shows UAC dialog, then exits this process
        else:
            print("   Continuing without elevation — WACK will be skipped.")

    # 2. Path verification
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

    msix_file = None
    if build_windows():
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
        exe_path = (
            Path(RUST_PROJECT_DIR) / "target" / "release" / "ScanCodeSyncDesktop.exe"
        )
        if exe_path.exists():
            msix_file = build_msix(new_version, exe_path)
        else:
            print(f"⚠️  Could not find .exe at {exe_path} — skipping MSIX.")
    else:
        print("⏭️  Skipping Windows build.")

    # 5. Move files into their version-scoped folder
    dest_path = Path(WEBSITE_DIR) / new_version
    dest_path.mkdir(parents=True, exist_ok=True)
    moved_files = []
    dmg_path = Path(RUST_PROJECT_DIR) / "ScanCodeSync.dmg"
    # Linux build writes ScanCodeSync.flatpak to the workspace root by default.
    fp_candidates = [
        Path("ScanCodeSync.flatpak"),
        Path(RUST_PROJECT_DIR) / "ScanCodeSync.flatpak",
    ]
    if os.path.exists(dmg_path):
        os.remove(dmg_path)
    if build_macos():
        script_dir = os.path.dirname(os.path.abspath(__file__))
        # Change the working directory to the script's folder
        os.chdir(script_dir)
        if dmg_path.exists():
            final_file = dest_path / f"ScanCodeSync-{new_version}.dmg"
            shutil.copy2(
                dmg_path,
                final_file,
            )
            moved_files.append(f"ScanCodeSync-{new_version}.dmg")

    for fp_path in fp_candidates:
        if fp_path.exists():
            os.remove(fp_path)
    if build_linux():
        script_dir = os.path.dirname(os.path.abspath(__file__))
        # Change the working directory to the script's folder
        os.chdir(script_dir)
        built_fp = next((p for p in fp_candidates if p.exists()), None)
        if built_fp:
            final_file = dest_path / f"ScanCodeSync-{new_version}.flatpak"
            shutil.copy2(
                built_fp,
                final_file,
            )
            moved_files.append(f"ScanCodeSync-{new_version}.flatpak")
            print(f"  -> Moved: {final_file.name}")
        else:
            print("⚠️  Linux build completed but no .flatpak artifact was found.")

    wix_path = Path(RUST_PROJECT_DIR) / "target" / "wix"

    for file in wix_path.glob("*.msi"):
        # Skip stale artifacts from previous builds that don't belong to this version
        if new_version not in file.name:
            print(f"  -- Skipping stale artifact: {file.name}")
            continue
        shutil.copy2(file, dest_path / file.name)
        moved_files.append(file.name)
        print(f"  -> Moved: {file.name}")

    if msix_file and msix_file.exists():
        shutil.copy2(msix_file, dest_path / msix_file.name)
        moved_files.append(msix_file.name)
        print(f"  -> Moved: {msix_file.name}")
        print(f"\n💡 Upload the .msix to Partner Center: https://partner.microsoft.com")

    # Run WACK on whichever .msix ended up in the version folder.
    # This works even if makeappx was skipped, as long as an .msix is present.
    msix_in_dest = list(dest_path.glob("*.msix"))
    if msix_in_dest:
        run_wack(msix_in_dest[0], dest_path)
    else:
        print("⚠️  No .msix found in output folder — skipping WACK.")

    if moved_files:
        # 6. Generate the update manifest (latest.json) from the artifacts now on disk
        print(f"\n📋 Generating update manifest …")
        generate_update_manifest(new_version, dest_path, WEBSITE_DIR)

        # 7. Regenerate HTML from disk — picks up all versions, not just this one
        update_releases_html(WEBSITE_DIR)
        print(f"\n🎉 Success! Files are in {dest_path}")
        if input("Create a GitHub release? [y/n]: ").strip().lower() in (
            "y",
            "yes",
            "",
        ):
            create_github_release(new_version, moved_files, dest_path)
        os.startfile(os.path.abspath(RELEASES_HTML_PATH))
    else:
        print("⚠️ No files were found to move.")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        input(e)
