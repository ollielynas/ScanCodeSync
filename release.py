import ctypes
import os
import re
import shutil
import subprocess
import sys
from datetime import datetime
from pathlib import Path

import toml

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
    """Returns True if the directory name looks like a semver string (e.g. 0.0.2)."""
    return path.is_dir() and bool(re.fullmatch(r"\d+\.\d+\.\d+", path.name))


def version_sort_key(version_str: str):
    """Converts '1.2.3' into (1, 2, 3) for correct numeric sorting."""
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
            if f.is_file() and f.suffix.lower() in {".msi", ".msix", ".dmg"}
        )

        if files:
            releases.append({"version": entry.name, "files": files})

    # Sort newest version first
    releases.sort(key=lambda r: version_sort_key(r["version"]), reverse=True)
    return releases


def render_release_card(release: dict, is_latest: bool) -> str:
    """Renders a single version card as an HTML string."""
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
        print("⚠️  No release directories found — releases.html not updated.")
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
        <p>Latest version: {latest_version} &mdash; page updated {date_str}</p>
        {cards_html}
    </div>
</body>
</html>
"""
    releases_path = os.path.join(website_dir, "releases.html")
    with open(releases_path, "w") as f:
        f.write(content)
    print(f"✅ Generated {releases_path} ({len(releases)} version(s) listed)")


def build_msix(version, exe_path):
    if not os.path.exists(MAKEAPPX_PATH):
        print(f"⚠️  makeappx.exe not found at {MAKEAPPX_PATH} — skipping MSIX build.")
        print("    Install the Windows SDK and update MAKEAPPX_PATH in this script.")
        return None

    print(f"📦 Building MSIX for v{version}...")

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
        print(
            "⚠️  No Assets folder found — MSIX may fail Store validation without icons."
        )

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


def build_linux(): ...
def build_macos():
    if input("would you like to build for MacOS? [y/n]") in ["n", "N"]:
        return False
    return macos.run_build_rust_over_ssh()


def run_wack(msix_path: Path, report_dir: Path) -> Path | None:
    """
    Runs the Windows App Certification Kit silently against an MSIX file
    and writes the XML report into report_dir.

    appcert.exe requires elevation (run this script as Administrator) and
    cannot run inside a remote desktop / headless session — it needs an
    interactive desktop to launch the tested app.

    Returns the Path to the report file, or None on failure.
    """
    if not os.path.exists(WACK_PATH):
        print(f"⚠️  appcert.exe not found at {WACK_PATH} — skipping WACK.")
        print(
            "    Install the Windows App Certification Kit via the Windows SDK installer."
        )
        return None

    report_path = report_dir / f"WACKReport_{msix_path.stem}.xml"

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
                print(f"✅ WACK result: PASS  — {report_path.name}")
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

    # 5. Move files into their version-scoped folder
    dest_path = Path(WEBSITE_DIR) / new_version
    dest_path.mkdir(parents=True, exist_ok=True)
    moved_files = []
    dmg_path = Path(RUST_PROJECT_DIR) / "ScanCodeSync.dmg"
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
        # 6. Regenerate HTML from disk — picks up all versions, not just this one
        update_releases_html(WEBSITE_DIR)
        print(f"\n🎉 Success! Files are in {dest_path}")
        os.startfile(os.path.abspath(RELEASES_HTML_PATH))
    else:
        print("⚠️ No files were found to move.")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        input(e)
