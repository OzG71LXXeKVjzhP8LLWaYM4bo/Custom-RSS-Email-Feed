#!/bin/bash
set -e

echo "Building release binary..."
cargo build --release

echo "Installing systemd units..."
mkdir -p ~/.config/systemd/user/
cp systemd/rss-market-digest.service ~/.config/systemd/user/
cp systemd/rss-market-digest.timer ~/.config/systemd/user/

echo "Reloading systemd..."
systemctl --user daemon-reload

echo "Enabling and starting timer..."
systemctl --user enable rss-market-digest.timer
systemctl --user start rss-market-digest.timer

echo ""
echo "Done! Timer is now active."
echo ""
echo "Useful commands:"
echo "  systemctl --user status rss-market-digest.timer   # Check timer status"
echo "  systemctl --user list-timers                      # List all timers"
echo "  systemctl --user start rss-market-digest.service  # Run manually"
echo "  journalctl --user -u rss-market-digest.service    # View logs"
