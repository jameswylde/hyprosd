#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$repo_root"
cargo build --release

install_dir="${HOME}/.local/bin"
mkdir -p "$install_dir"
cp -f "target/release/hyprosd" "${install_dir}/hyprosd"

cat > "${install_dir}/hyprosd-daemon" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

exec "$(dirname "${BASH_SOURCE[0]}")/hyprosd" daemon
EOF
chmod +x "${install_dir}/hyprosd-daemon"

echo "Installed to ${install_dir}/hyprosd"
echo "Installed daemon launcher to ${install_dir}/hyprosd-daemon"

case ":${PATH}:" in
  *":${install_dir}:"*) ;;
  *)
    echo
    echo "Warning: ${install_dir} is not in PATH for this shell."
    echo "Use this in Hyprland:"
    echo "  exec-once = ${install_dir}/hyprosd-daemon"
    ;;
esac
