cask "scancodesync" do
  # Fetch latest version info from update endpoint
  json = begin
    require "open-uri"
    require "json"
    JSON.parse(URI.open("https://sync-home.ollielynas.com/latest.json").read)
  end

  platform_key = "macos"
  platform     = json["platforms"][platform_key]

  version json["version"]
  sha256 platform["signature"].sub(/^sha256:/, "")
  url platform["url"]

  name "ScanCodeSync"
  desc "Scan code sync desktop application"
  homepage "https://sync-home.ollielynas.com"

  livecheck do
    url "https://sync-home.ollielynas.com/latest.json"
    strategy :json do |json|
      json["version"]
    end
  end

  app "ScanCodeSync.app"

  postflight do
    system_command "/usr/bin/xattr",
                   args: ["-dr", "com.apple.quarantine", "#{appdir}/ScanCodeSync.app"],
                   sudo: false
  end

  uninstall quit: "com.ollielynas.scancodesync"

  zap trash: [
    "~/Library/Application Support/ScanCodeSync",
    "~/Library/Preferences/com.ollielynas.scancodesync.plist",
    "~/Library/Logs/ScanCodeSync",
  ]
end
