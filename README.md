# top-bounce

A Rust reimplementation of [TopBounce](https://github.com/joshvoigts/TopBounce) — a macOS menu bar blocker that prevents accidental menu bar activation by clamping the cursor position when it reaches the top of the screen.

## Installing

```bash
cargo install top-bounce
```

## Running at Login

To run `top-bounce` automatically in the background at startup, create a LaunchAgent:

```bash
mkdir -p ~/Library/LaunchAgents
cat > ~/Library/LaunchAgents/com.joshvoigts.topbounce.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.joshvoigts.topbounce</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/joshvoigts/.cargo/bin/top-bounce</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
EOF
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/com.joshvoigts.topbounce.plist
```

### Stop and remove from login

```bash
launchctl bootout gui/$(id -u) ~/Library/LaunchAgents/com.joshvoigts.topbounce.plist
```

### Uninstall

```bash
rm ~/Library/LaunchAgents/com.joshvoigts.topbounce.plist
```

## Notes

- Requires **Accessibility permissions**:
  1. Open **System Settings → Privacy & Security → Accessibility**
  2. Click the **+** button
  3. Navigate to `~/.cargo/bin/top-bounce` and add it
  4. Go to "Login Items & Extensions"
  5. Ensure top-bounce is enabled (sometimes toggling it can help)
- Hold **Shift** while moving the mouse to temporarily bypass the restriction.
- The top limit is set to 8 pixels by default.
- Does not seem to work properly on macs with the "notch".

## Troubleshooting

Go to console app and type "bounce" and then enter in the search box.
