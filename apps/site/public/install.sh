#!/bin/sh
# Installs the repot command-line tool from a GitHub release:
#
#   curl -fsSL https://junjuny0227.github.io/Repot/install.sh | sh
#
# It downloads the archive for this OS and CPU, checks it against the release's .sha256 file
# and copies `repot` into ~/.local/bin. Nothing else on the machine is changed: no sudo, no
# shell profile edits. Read it first: https://github.com/junjuny0227/Repot/blob/main/apps/site/public/install.sh
#
# Settings (environment variables):
#   REPOT_VERSION       version to install, e.g. 0.9.0 (default: the latest release)
#   REPOT_INSTALL_DIR   where to put repot (default: ~/.local/bin)
#
# The source of this file is apps/site/public/install.sh; the site publishes it as is.

set -eu

# Everything runs from main(), called on the last line, so a download cut short by the
# network cannot run half a script.
main() {
  releases="${REPOT_RELEASES_URL:-https://github.com/junjuny0227/Repot/releases}"
  install_dir="${REPOT_INSTALL_DIR:-$HOME/.local/bin}"

  need curl
  need tar
  need uname
  need mktemp

  target=$(detect_target)
  version=$(resolve_version "$releases")
  name="repot-cli-$version-$target"
  url="$releases/download/v$version/$name.tar.gz"

  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT

  say "Downloading repot $version for $target"
  curl -fsSL -o "$tmp/$name.tar.gz" "$url" || fail "could not download $url"
  curl -fsSL -o "$tmp/$name.tar.gz.sha256" "$url.sha256" || fail "could not download $url.sha256"

  expected=$(cut -d ' ' -f 1 <"$tmp/$name.tar.gz.sha256")
  actual=$(sha256 "$tmp/$name.tar.gz")
  [ -n "$expected" ] && [ "$expected" = "$actual" ] ||
    fail "checksum mismatch for $name.tar.gz (expected $expected, got $actual); nothing was installed"

  tar -xzf "$tmp/$name.tar.gz" -C "$tmp"
  [ -f "$tmp/$name/repot" ] || fail "$name.tar.gz has no repot in it; nothing was installed"

  mkdir -p "$install_dir"
  # Copy next to the target, then rename: a running repot is never left half-written.
  cp "$tmp/$name/repot" "$install_dir/.repot.new"
  chmod 755 "$install_dir/.repot.new"
  mv -f "$install_dir/.repot.new" "$install_dir/repot"

  say "Installed $("$install_dir/repot" --version) to $install_dir/repot"
  path_hint "$install_dir"
}

say() { printf '%s\n' "$1"; }

fail() {
  printf 'repot install: %s\n' "$1" >&2
  exit 1
}

need() {
  command -v "$1" >/dev/null 2>&1 || fail "needs $1, which was not found"
}

# Release archives exist for these targets only (see release.yml).
detect_target() {
  case "$(uname -s)" in
  Linux) os=unknown-linux-gnu ;;
  Darwin) os=apple-darwin ;;
  *) fail "no prebuilt repot for $(uname -s); see https://github.com/junjuny0227/Repot/blob/main/crates/cli/README.md" ;;
  esac
  case "$(uname -m)" in
  x86_64 | amd64) arch=x86_64 ;;
  arm64 | aarch64) arch=aarch64 ;;
  *) arch=$(uname -m) ;;
  esac
  case "$arch-$os" in
  x86_64-unknown-linux-gnu | x86_64-apple-darwin | aarch64-apple-darwin) printf '%s\n' "$arch-$os" ;;
  *) fail "no prebuilt repot for $arch-$os yet; with Rust, run: cargo install --git https://github.com/junjuny0227/Repot repot-cli" ;;
  esac
}

# REPOT_VERSION, or the tag that /releases/latest redirects to (no API call, no token).
resolve_version() {
  if [ -n "${REPOT_VERSION:-}" ]; then
    printf '%s\n' "${REPOT_VERSION#v}"
    return
  fi
  latest=$(curl -fsSLI -o /dev/null -w '%{url_effective}' "$1/latest") ||
    fail "could not reach $1/latest"
  case "$latest" in
  */tag/v*) printf '%s\n' "${latest##*/tag/v}" ;;
  *) fail "could not find the latest release at $1/latest" ;;
  esac
}

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d ' ' -f 1
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | cut -d ' ' -f 1
  else
    fail "needs sha256sum or shasum to check the download"
  fi
}

# Says how to put the folder on PATH; changing shell profiles is left to the user.
path_hint() {
  case ":$PATH:" in
  *":$1:"*) return ;;
  esac
  # The ~ is printed for the user to type, not expanded here.
  # shellcheck disable=SC2088
  case "$(basename "${SHELL:-sh}")" in
  zsh) profile='~/.zshrc' ;;
  bash) profile='~/.bashrc' ;;
  fish)
    say ""
    say "$1 is not on your PATH. To add it, run: fish_add_path $1"
    return
    ;;
  *) profile='~/.profile' ;;
  esac
  say ""
  say "$1 is not on your PATH. To add it, run:"
  say "  echo 'export PATH=\"$1:\$PATH\"' >> $profile"
  say "then open a new terminal."
}

main "$@"
