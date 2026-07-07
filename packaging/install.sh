#!/bin/bash
# maccel install script.
#
# Builds the binary in release mode and installs:
#   /usr/local/bin/maccel                    (the daemon)
#   /etc/systemd/system/maccel.service       (systemd unit)
#   /etc/maccel/config.toml                  (default config, only if absent)
#
# After install:
#   sudo systemctl enable --now maccel
#
# Must be run from the repo root. Requires cargo and (for the systemd
# integration) a Linux host with systemd.

set -euo pipefail

PREFIX="${PREFIX:-/usr/local}"
CONFIG_DIR="${CONFIG_DIR:-/etc/maccel}"
UNIT_DIR="${UNIT_DIR:-/etc/systemd/system}"

echo "=== building release binary ==="
cargo build --release

BINARY="target/release/maccel"
if [[ ! -x "$BINARY" ]]; then
    echo "error: release binary not found at $BINARY" >&2
    exit 1
fi

echo "=== installing binary to ${PREFIX}/bin/maccel ==="
install -d "${PREFIX}/bin"
install -m 0755 "$BINARY" "${PREFIX}/bin/maccel"

echo "=== installing systemd unit to ${UNIT_DIR}/maccel.service ==="
if [[ -d "$UNIT_DIR" ]]; then
    install -m 0644 packaging/maccel.service "${UNIT_DIR}/maccel.service"
else
    echo "warning: ${UNIT_DIR} does not exist; skipping systemd unit install"
    echo "  (this is expected on non-systemd hosts)"
fi

echo "=== installing default config to ${CONFIG_DIR}/config.toml ==="
install -d "$CONFIG_DIR"
if [[ -f "${CONFIG_DIR}/config.toml" ]]; then
    echo "  config already exists; leaving it in place"
else
    install -m 0644 config/default.toml "${CONFIG_DIR}/config.toml"
fi

echo
echo "=== install complete ==="
echo
echo "Next steps:"
echo "  1. Add your user to the input group (or run the service as root):"
echo "       sudo usermod -aG input \$USER"
echo "  2. Ensure /dev/uinput is accessible. Either run as root, or add a udev rule:"
echo "       echo 'KERNEL==\"uinput\", GROUP=\"input\", MODE=\"0660\"' | \\"
echo "         sudo tee /etc/udev/rules.d/99-maccel.rules"
echo "       sudo udevadm control --reload-rules"
echo "  3. Reload systemd and enable the service:"
echo "       sudo systemctl daemon-reload"
echo "       sudo systemctl enable --now maccel"
echo "  4. Verify the daemon is running:"
echo "       sudo systemctl status maccel"
echo "       sudo journalctl -u maccel -f"
echo
echo "To uninstall:"
echo "  sudo systemctl disable --now maccel"
echo "  sudo rm ${PREFIX}/bin/maccel ${UNIT_DIR}/maccel.service"
echo "  sudo rm -rf ${CONFIG_DIR}"
