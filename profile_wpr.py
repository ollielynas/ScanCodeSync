#!/usr/bin/env python3
"""
Windows Performance Recorder profiling script.
Elevates to admin, records system performance, and saves to ETL file.
"""

import ctypes
import os
import sys
import subprocess
from datetime import datetime
from pathlib import Path


def is_admin() -> bool:
    """Check if running as administrator."""
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


def get_output_path() -> str:
    """Generate output path with timestamp."""
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_dir = Path("profiles")
    output_dir.mkdir(exist_ok=True)
    return str(output_dir / f"trace_{timestamp}.etl")


def start_wpr(profile: str = "GeneralProfile") -> bool:
    """Start WPR recording."""
    try:
        result = subprocess.run(
            ["wpr", "-start", profile],
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            print(f"❌ Failed to start WPR: {result.stderr}")
            return False
        print(f"✅ WPR started with profile: {profile}")
        return True
    except FileNotFoundError:
        print(
            "❌ wpr.exe not found. Please install Windows Performance Toolkit:"
        )
        print("   https://learn.microsoft.com/en-us/windows-hardware/get-started/adk-install")
        return False


def stop_wpr(output_path: str) -> bool:
    """Stop WPR recording and save to file."""
    try:
        result = subprocess.run(
            ["wpr", "-stop", output_path],
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            print(f"❌ Failed to stop WPR: {result.stderr}")
            return False
        print(f"✅ WPR stopped and saved to: {output_path}")
        return True
    except FileNotFoundError:
        print("❌ wpr.exe not found.")
        return False


def open_wpa(output_path: str) -> None:
    """Open the ETL file in Windows Performance Analyzer."""
    try:
        subprocess.Popen(["wpa", output_path])
        print(f"✅ Opening trace in WPA: {output_path}")
    except FileNotFoundError:
        print(
            f"⚠️  wpa.exe not found. You can open the trace manually:"
        )
        print(f"   wpa \"{output_path}\"")


def build_rust_project() -> bool:
    """Build the Rust project in release mode."""
    print("\n🔨 Building Rust project...\n")
    try:
        result = subprocess.run(
            ["cargo", "build", "--release", "-p", "ScanCodeSyncDesktop"],
            cwd="ScanCodeSyncDesktop",
        )
        if result.returncode != 0:
            print(f"\n❌ Build failed")
            return False
        print("\n✅ Build successful")
        return True
    except FileNotFoundError:
        print("❌ cargo not found. Is Rust installed?")
        return False


def find_app_executable() -> str:
    """Find ScanCodeSyncDesktop executable."""
    candidates = [
        "ScanCodeSyncDesktop/target/release/ScanCodeSyncDesktop.exe",
        "ScanCodeSyncDesktop/target/debug/ScanCodeSyncDesktop.exe",
    ]
    
    for candidate in candidates:
        if Path(candidate).exists():
            return str(Path(candidate).resolve())
    
    # Not found - return empty string
    return ""


def launch_app(app_path: str) -> bool:
    """Launch the app and wait for it to close."""
    print(f"\n🚀 Launching: {app_path}")
    try:
        process = subprocess.Popen(app_path)
        print("✅ App launched. Profiling in progress...")
        print("📊 Close the app when done profiling.\n")
        
        # Wait for app to exit
        process.wait()
        print("\n✅ App closed.")
        return True
    except Exception as e:
        print(f"❌ Failed to launch app: {e}")
        return False


def main():
    """Main profiling workflow."""
    print("=" * 60)
    print("Windows Performance Recorder - Profile ScanCodeSync")
    print("=" * 60)

    # Check for admin privileges
    if not is_admin():
        print("⚠️  This script requires administrator privileges.")
        print("🔄 Requesting elevation...\n")
        relaunch_as_admin()
        # relaunch_as_admin() exits, so we never reach here

    # Build the project
    if not build_rust_project():
        sys.exit(1)

    # Find the app executable
    app_path = find_app_executable()
    if not app_path:
        print("❌ Could not find ScanCodeSyncDesktop executable.")
        print("   Checked: ScanCodeSyncDesktop/target/release/ScanCodeSyncDesktop.exe")
        sys.exit(1)
    print(f"✅ Found executable: {app_path}")

    # Get output path
    output_path = get_output_path()
    print(f"📁 Output file: {output_path}")

    # Choose profile
    print("\nAvailable profiles:")
    print("  [1] GeneralProfile (default - captures everything)")
    print("  [2] CPU (CPU usage only)")
    print("  [3] DiskIO (Disk I/O only)")
    print("  [4] Memory (Memory allocation)")
    choice = input("Select profile [1]: ").strip() or "1"

    profile_map = {
        "1": "GeneralProfile",
        "2": "CPU",
        "3": "DiskIO",
        "4": "Memory",
    }
    profile = profile_map.get(choice, "GeneralProfile")

    # Start recording
    print(f"\n🎥 Starting WPR with {profile} profile...")
    if not start_wpr(profile):
        sys.exit(1)

    # Launch app
    if not launch_app(app_path):
        sys.exit(1)

    # Stop recording
    print("\n⏹️  Stopping WPR...")
    if not stop_wpr(output_path):
        sys.exit(1)

    # Offer to open in WPA
    print("\n" + "=" * 60)
    print("✅ Profiling complete!")
    print("=" * 60)
    open_wpa_choice = (
        input("\nOpen trace in Windows Performance Analyzer? [Y/n]: ")
        .strip()
        .lower()
    )
    if open_wpa_choice != "n":
        open_wpa(output_path)

    print(f"\n💡 To view later, run: wpa \"{output_path}\"")
    print("\nDone!")


if __name__ == "__main__":
    main()
