from __future__ import annotations

import argparse
import shlex
import subprocess
import textwrap
from pathlib import Path

DEFAULT_APP_ID = "com.OllieLynas.ScanCodeSync"
DEFAULT_RUNTIME_VERSION = "24.08"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build a Flatpak bundle for ScanCodeSync using WSL."
    )
    parser.add_argument(
        "--app-id",
        default=DEFAULT_APP_ID,
        help="Flatpak application ID and manifest filename prefix.",
    )
    parser.add_argument(
        "--runtime-version",
        default=DEFAULT_RUNTIME_VERSION,
        help="Freedesktop runtime version to install (example: 23.08).",
    )
    parser.add_argument(
        "--source-dir",
        default=str(Path(__file__).resolve().parent),
        help="Windows path to the Flatpak project source folder.",
    )
    parser.add_argument(
        "--output-dir",
        default=str(Path(__file__).resolve().parent),
        help="Windows path where ScanCodeSync.flatpak will be copied.",
    )
    parser.add_argument(
        "--keep-build-dir",
        action="store_true",
        help="Keep the Linux temporary build directory for debugging.",
    )
    return parser.parse_args()


def quote_for_bash(value: str) -> str:
    return shlex.quote(value)


def single_quote_for_bash(value: str) -> str:
    return "'" + value.replace("'", "'\"'\"'") + "'"


def debug_log(message: str) -> None:
    print(f"[linux-build-debug] {message}")


def windows_path_to_wsl(path: Path) -> str:
    original = str(path)
    as_forward_slash = original.replace("\\", "/")

    attempts = [
        original,
        as_forward_slash,
    ]

    debug_log(f"Converting Windows path: {original}")

    for idx, candidate in enumerate(attempts, start=1):
        debug_log(f"Attempt {idx}: wslpath input = {candidate}")
        result = subprocess.run(
            ["wsl", "wslpath", "-u", candidate],
            capture_output=True,
            text=True,
            check=False,
        )

        stdout = result.stdout.strip()
        stderr = result.stderr.strip()
        debug_log(f"Attempt {idx}: return code = {result.returncode}")
        if stdout:
            debug_log(f"Attempt {idx}: stdout = {stdout}")
        if stderr:
            debug_log(f"Attempt {idx}: stderr = {stderr}")

        if result.returncode == 0 and stdout:
            return stdout

    drive = path.drive.rstrip(":")
    if drive and path.anchor:
        relative = str(path).replace(path.anchor, "", 1).replace("\\", "/")
        manual = f"/mnt/{drive.lower()}/{relative}"
        debug_log(f"Fallback manual path candidate = {manual}")
        return manual

    raise RuntimeError(f"Failed to convert Windows path to WSL path: {path}")


def build_wsl_script(
    app_id: str,
    runtime_version: str,
    source_dir_linux: str,
    output_dir_linux: str,
    keep_build_dir: bool,
) -> str:
    cleanup_block = "" if keep_build_dir else 'rm -rf "$BUILD_DIR"\n'

    return textwrap.dedent(
        f"""
        set -euo pipefail

        APP_ID={single_quote_for_bash(app_id)}
        RUNTIME_VERSION={single_quote_for_bash(runtime_version)}
        SOURCE_DIR_LINUX={single_quote_for_bash(source_dir_linux)}
        OUTPUT_DIR_LINUX={single_quote_for_bash(output_dir_linux)}
        BUILD_DIR="/home/$USER/flatpak_build_temp_${{APP_ID##*.}}"

        echo "--- Starting Linux Flatpak build ---"
        echo "Raw APP_ID assignment: $APP_ID"
        echo "Raw SOURCE_DIR_LINUX assignment: $SOURCE_DIR_LINUX"
        echo "Raw OUTPUT_DIR_LINUX assignment: $OUTPUT_DIR_LINUX"
        echo "Source: $SOURCE_DIR_LINUX"
        echo "Output: $OUTPUT_DIR_LINUX"

        if [ -z "$SOURCE_DIR_LINUX" ] || [ -z "$OUTPUT_DIR_LINUX" ]; then
            echo "ERROR: missing Linux paths"
            exit 1
        fi

        if [ ! -d "$SOURCE_DIR_LINUX" ]; then
            echo "ERROR: source directory does not exist: $SOURCE_DIR_LINUX"
            exit 1
        fi

        rm -rf "$BUILD_DIR"
        mkdir -p "$BUILD_DIR"

        if command -v rsync >/dev/null 2>&1; then
            rsync -a --delete \
                --exclude '.git/' \
                --exclude 'target/' \
                --exclude 'build-dir/' \
                "$SOURCE_DIR_LINUX/" "$BUILD_DIR/"
        else
            cp -a "$SOURCE_DIR_LINUX/." "$BUILD_DIR/"
        fi

        cd "$BUILD_DIR"

        MANIFEST="$APP_ID.json"
        if [ ! -f "$MANIFEST" ]; then
            echo "ERROR: manifest not found: $MANIFEST"
            echo "Files in build directory:"
            ls -la
            exit 1
        fi

        if ! command -v flatpak-builder >/dev/null 2>&1; then
            echo "--- Installing flatpak tools ---"
            sudo apt update
            sudo apt install -y flatpak flatpak-builder build-essential
        fi

        flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
        flatpak install --user -y flathub \
            "org.freedesktop.Sdk//$RUNTIME_VERSION" \
            "org.freedesktop.Platform//$RUNTIME_VERSION" \
            "org.freedesktop.Sdk.Extension.rust-stable//$RUNTIME_VERSION"

        echo "--- Building Flatpak ---"
        flatpak-builder \
            --force-clean \
            --user \
            --install-deps-from=flathub \
            --repo=repo \
            build-dir \
            "$MANIFEST"

        BUNDLE_NAME="ScanCodeSync.flatpak"
        flatpak build-bundle repo "$BUNDLE_NAME" "$APP_ID"

        mkdir -p "$OUTPUT_DIR_LINUX"
        cp "$BUNDLE_NAME" "$OUTPUT_DIR_LINUX/"
        {cleanup_block}echo "--- SUCCESS: copied $BUNDLE_NAME to $OUTPUT_DIR_LINUX ---"
        """
    ).strip()


def run_wsl_build(script: str) -> None:
    debug_log("Executing build script via stdin (wsl bash -s)")
    normalized_script = script.replace("\r\n", "\n").replace("\r", "\n")
    subprocess.run(
        ["wsl", "bash", "-s"],
        input=normalized_script.encode("utf-8"),
        check=True,
    )


def build_for_linux_using_wsl():
    args = parse_args()

    source_dir = Path(args.source_dir).resolve()
    output_dir = Path(args.output_dir).resolve()
    source_dir_linux = windows_path_to_wsl(source_dir)
    output_dir_linux = windows_path_to_wsl(output_dir)

    script = build_wsl_script(
        app_id=args.app_id,
        runtime_version=args.runtime_version,
        source_dir_linux=source_dir_linux,
        output_dir_linux=output_dir_linux,
        keep_build_dir=args.keep_build_dir,
    )

    print("--- Running WSL Flatpak build ---")
    print(f"App ID: {args.app_id}")
    print(f"Source dir: {source_dir}")
    print(f"Output dir: {output_dir}")
    debug_log(f"Resolved WSL source dir: {source_dir_linux}")
    debug_log(f"Resolved WSL output dir: {output_dir_linux}")

    run_wsl_build(script)
    return True


if __name__ == "__main__":
    build_for_linux_using_wsl()
