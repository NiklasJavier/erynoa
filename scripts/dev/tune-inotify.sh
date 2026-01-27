#!/bin/bash
# ============================================================================
# Linux Kernel Tuning für File Watchers (inotify)
# ============================================================================
# Bei großen Projekten (Rust target + node_modules) können die Standard-Limits
# überschritten werden, was dazu führt, dass Hot-Reloading aufhört zu funktionieren.
#
# Dieses Script erhöht die Limits auf dem Host-System.
# ============================================================================

set -e

echo "🔧 Checking inotify limits..."

CURRENT_WATCHES=$(cat /proc/sys/fs/inotify/max_user_watches 2>/dev/null || echo "unknown")
CURRENT_INSTANCES=$(cat /proc/sys/fs/inotify/max_user_instances 2>/dev/null || echo "unknown")

echo "   Current max_user_watches: $CURRENT_WATCHES"
echo "   Current max_user_instances: $CURRENT_INSTANCES"

# Empfohlene Werte für große Monorepos
RECOMMENDED_WATCHES=524288
RECOMMENDED_INSTANCES=512

if [ "$CURRENT_WATCHES" != "unknown" ] && [ "$CURRENT_WATCHES" -lt "$RECOMMENDED_WATCHES" ]; then
    echo ""
    echo "⚠️  inotify limits sind zu niedrig für große Projekte!"
    echo ""
    echo "Empfohlene Limits setzen (erfordert sudo):"
    echo ""
    echo "  # Temporär (bis zum Neustart):"
    echo "  sudo sysctl fs.inotify.max_user_watches=$RECOMMENDED_WATCHES"
    echo "  sudo sysctl fs.inotify.max_user_instances=$RECOMMENDED_INSTANCES"
    echo ""
    echo "  # Permanent (in /etc/sysctl.conf oder /etc/sysctl.d/99-inotify.conf):"
    echo "  fs.inotify.max_user_watches=$RECOMMENDED_WATCHES"
    echo "  fs.inotify.max_user_instances=$RECOMMENDED_INSTANCES"
    echo ""
    
    read -p "Möchtest du die Limits jetzt temporär setzen? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        sudo sysctl fs.inotify.max_user_watches=$RECOMMENDED_WATCHES
        sudo sysctl fs.inotify.max_user_instances=$RECOMMENDED_INSTANCES
        echo "✅ Limits wurden temporär erhöht!"
    fi
else
    echo "✅ inotify limits sind ausreichend!"
fi
