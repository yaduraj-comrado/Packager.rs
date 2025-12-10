#!/bin/bash
# Script to list all available packages from apt, snap, and flatpak

{
    # APT: List ALL packages (~115k packages)
    apt-cache pkgnames 2>/dev/null | sed 's/^/[apt] /' &
    
    # FLATPAK: List ALL packages from all remotes (~3k+ packages)
    {
        for remote in $(flatpak remotes --columns=name 2>/dev/null); do
            flatpak remote-ls "$remote" --app --columns=application,name 2>/dev/null | \
                awk -v remote="$remote" '{if(NF>=2) { app=$1; $1=""; print "[flatpak-"remote"] " app $0 }}' 
        done
        
        flatpak list --app --columns=application,name 2>/dev/null | \
            awk '{if(NF>=2) { app=$1; $1=""; print "[flatpak-installed] " app $0 }}'
    } | sort -u &
    
    # SNAP: No "list all" API exists - search broadly to get coverage
    # Note: This is a limitation of Snap's API design
    {
        snap list 2>/dev/null | tail -n +2 | awk '{print "[snap-installed] " $1}'
        
        # Parallel snap searches for better performance
        # Search most common terms to get reasonable coverage quickly
        for term in "" app browser editor media video music photo image game dev code \
                    tool server database network security office productivity \
                    chat email note calendar file cloud backup system util; do
            snap find "$term" 2>/dev/null | tail -n +2 | awk 'NF > 0 && !/^[[:space:]]/ {print "[snap] " $1}' &
        done
        wait
    } | sort -u &
    
    wait
} 2>/dev/null
