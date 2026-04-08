# top-bounce

A Rust reimplementation of [TopBounce](https://github.com/joshvoigts/TopBounce) — a macOS menu bar blocker that prevents accidental menu bar activation by clamping the cursor position when it reaches the top of the screen.

## Installing

```bash
cargo install --path .
```

## Running at Login

To run `top-bounce` automatically in the background at startup, create a LaunchAgent:

```bash
mkdir -p ~/Library/LaunchAgents
cat > ~/Library/LaunchAgents/com.topbounce.agent.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.topbounce.agent</string>
    <key>ProgramArguments</key>
    <array>
        <string>~/.cargo/bin/top-bounce</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>Hidden</key>
    <true/>
</dict>
</plist>
EOF
launchctl load ~/Library/LaunchAgents/com.topbounce.agent.plist
```

## Notes

- Requires **Accessibility permissions** (System Settings → Privacy & Security → Accessibility).
- Hold **Shift** while moving the mouse to temporarily bypass the restriction.
- The top limit is set to 8 pixels by default.
