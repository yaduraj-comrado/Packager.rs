#!/bin/bash
# Parse the selected package line and install it
LINE="$1"

# Extract package manager type and package name
if [[ "$LINE" =~ ^\[apt\][[:space:]]*([^[:space:]]+) ]]; then
    PKG="${BASH_REMATCH[1]}"
    echo "Installing $PKG via apt..."
    sudo apt install -y "$PKG"
elif [[ "$LINE" =~ ^\[flatpak-([^]]+)\][[:space:]]*([^[:space:]]+) ]]; then
    REMOTE="${BASH_REMATCH[1]}"
    PKG="${BASH_REMATCH[2]}"
    echo "Installing $PKG via flatpak..."
    if [[ "$REMOTE" == "installed" ]]; then
        echo "$PKG is already installed"
    else
        flatpak install -y "$REMOTE" "$PKG"
    fi
elif [[ "$LINE" =~ ^\[snap(-installed)?\][[:space:]]*([^[:space:]]+) ]]; then
    PKG="${BASH_REMATCH[2]}"
    if [[ -n "${BASH_REMATCH[1]}" ]]; then
        echo "$PKG is already installed"
    else
        echo "Installing $PKG via snap..."
        sudo snap install "$PKG"
    fi
elif [[ "$LINE" =~ ^\[cargo(-installed)?\][[:space:]]*([^[:space:]]+) ]]; then
    PKG="${BASH_REMATCH[2]}"
    if [[ -n "${BASH_REMATCH[1]}" ]]; then
        echo "$PKG is already installed"
    else
        echo "Installing $PKG via cargo..."
        cargo install "$PKG"
    fi
elif [[ "$LINE" =~ ^\[npm(-installed)?\][[:space:]]*([^[:space:]]+) ]]; then
    PKG="${BASH_REMATCH[2]}"
    if [[ -n "${BASH_REMATCH[1]}" ]]; then
        echo "$PKG is already installed"
    else
        echo "Installing $PKG via npm (globally)..."
        npm install -g "$PKG"
    fi
else
    echo "Could not parse package from: $LINE"
    exit 1
fi