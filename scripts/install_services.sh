#!/bin/bash
set -e

# Install services
echo "Installing systemd services..."
mkdir -p ~/.config/systemd/user/
cp systemd/*.service ~/.config/systemd/user/
cp systemd/*.timer ~/.config/systemd/user/

# Reload and enable
systemctl --user daemon-reload
systemctl --user enable --now vlog.service
systemctl --user enable --now vlog-daily.timer

echo "Services installed and started."
systemctl --user status vlog.service --no-pager
