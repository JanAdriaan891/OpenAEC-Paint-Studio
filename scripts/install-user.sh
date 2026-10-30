#!/usr/bin/env bash
# Reversible per-user Linux installation; existing default image viewers are preserved.
set -euo pipefail
package="$(realpath "${1:?Usage: install-user.sh path/to/installer.deb}")"
[[ "$(dpkg-deb -f "$package" Package)" == open-aec-paint-studio ]] || { echo 'Unexpected package' >&2; exit 1; }
version="$(dpkg-deb -f "$package" Version)"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo 'Unexpected version' >&2; exit 1; }
appdir="${XDG_DATA_HOME:-$HOME/.local/share}/openaec-paint-studio"
bindir="$HOME/.local/bin"
mkdir -p "$appdir/releases" "$bindir"
stage="$(mktemp -d "$appdir/.stage.XXXXXX")"
trap 'rm -rf -- "$stage"' EXIT
dpkg-deb --extract "$package" "$stage"
for bin in openaec-paint-studio paint-api paint-mcp; do
  [[ -x "$stage/usr/bin/$bin" ]] || { echo "Missing executable: $bin" >&2; exit 1; }
  missing="$(ldd "$stage/usr/bin/$bin" 2>&1)"
  if [[ "$missing" == *'not found'* ]]; then printf '%s\n' "$missing" >&2; exit 1; fi
done
fingerprint="$(sha256sum "$package")";fingerprint="${fingerprint:0:12}"
destination="$appdir/releases/$version-$fingerprint"
if [[ ! -e "$destination" ]]; then mv "$stage" "$destination";fi
ln -sfn "$destination" "$appdir/current.next"
mv -Tf "$appdir/current.next" "$appdir/current"
for bin in openaec-paint-studio paint-api paint-mcp; do
  printf '#!/usr/bin/env bash\nexec %q "$@"\n' "$appdir/current/usr/bin/$bin" > "$bindir/$bin"
  chmod 755 "$bindir/$bin"
done
data="${XDG_DATA_HOME:-$HOME/.local/share}"
mkdir -p "$data/applications"
# Escape the executable for the desktop-entry quoted-string grammar.
launcher="$bindir/openaec-paint-studio";launcher="${launcher//\\/\\\\}";launcher="${launcher//\"/\\\"}"
while IFS= read -r line;do
  case "$line" in Exec=*) printf 'Exec="%s" %%f\n' "$launcher";; *) printf '%s\n' "$line";; esac
done < "$destination/usr/share/applications/org.openaec.paint-studio.desktop" > "$data/applications/org.openaec.paint-studio.desktop"
install -Dm644 "$destination/usr/share/icons/hicolor/128x128/apps/org.openaec.paint-studio.png" "$data/icons/hicolor/128x128/apps/org.openaec.paint-studio.png"
install -Dm644 "$destination/usr/share/mime/packages/openaec-paint-studio.xml" "$data/mime/packages/openaec-paint-studio.xml"
command -v update-desktop-database >/dev/null && update-desktop-database "$data/applications" || true
command -v update-mime-database >/dev/null && update-mime-database "$data/mime" || true
printf 'Installed OpenAEC Paint Studio %s at %s\nPrevious releases remain in %s/releases.\n' "$version" "$destination" "$appdir"
