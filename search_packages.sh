#!/bin/bash
# Script to search packages across apt, snap, and flatpak
# Usage: search_packages.sh <query>

query="$1"

if [ -z "$query" ]; then
    exit 0
fi

# Search apt
apt-cache search "$query" 2>/dev/null | sed 's/^/[apt] /' &

# Search snap (extract package name only, skip summary lines)
snap find "$query" 2>/dev/null | tail -n +2 | awk 'NF > 0 && !/^[[:space:]]/ {print "[snap] " $1}' &

# Search flatpak
flatpak search "$query" 2>/dev/null | tail -n +2 | grep -v '^$' | sed 's/^/[flatpak] /' &

wait
