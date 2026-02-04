# 🔧 Migration Scripts: Automatisierbare Schritte

> **Teil von:** Projekt Pluto

---

## 1. Setup-Script

```bash
#!/bin/bash
# scripts/pluto-setup.sh
# Erstellt die neue Verzeichnisstruktur

set -e

cd /Users/niklas/Development/30_Development_Code/31_git_ownbusiness/erynoa/erynoa-main/backend/src

echo "🌑 Projekt Pluto: Erstelle neue Struktur..."

# Nervous System
mkdir -p nervous_system/{event_sourcing,merkle,components,graph,infrastructure}

# Synapses
mkdir -p synapses/adapters

# Realm
mkdir -p realm/{sharding,quota,gateway,saga}

# Storage
mkdir -p storage/{kv,event_store,identity_store,trust_store,content_store,archive,realm,blueprint,metrics}

# Identity
mkdir -p identity

# Engines
mkdir -p engines

echo "✅ Verzeichnisse erstellt"

# Basis mod.rs Dateien erstellen
for dir in nervous_system synapses realm storage identity engines; do
    if [ ! -f "$dir/mod.rs" ]; then
        echo "//! $dir module" > "$dir/mod.rs"
        echo "Created $dir/mod.rs"
    fi
done

echo "🌑 Projekt Pluto Setup abgeschlossen!"
```

---

## 2. Extraktion: EventBus

```bash
#!/bin/bash
# scripts/extract-event-bus.sh

SOURCE="core/state.rs"
TARGET="nervous_system/infrastructure/event_bus.rs"

echo "Extrahiere EventBus..."

# Zeilen 39-400 aus state.rs nach event_bus.rs
sed -n '39,400p' "$SOURCE" > "$TARGET"

# Header hinzufügen
cat << 'EOF' | cat - "$TARGET" > temp && mv temp "$TARGET"
//! # Event Bus
//!
//! P2P ↔ Core Kommunikation über bounded Queues.
//!
//! Extrahiert aus state.rs (Zeilen 39-400)

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

use crate::domain::unified::primitives::UniversalId;
use crate::domain::unified::system::EventPriority;

EOF

echo "✅ EventBus extrahiert nach $TARGET"
```

---

## 3. Cargo.toml Backup

```bash
#!/bin/bash
# scripts/backup-before-refactor.sh

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="../backups/pluto_$TIMESTAMP"

echo "Erstelle Backup: $BACKUP_DIR"

mkdir -p "$BACKUP_DIR"
cp -r backend/src "$BACKUP_DIR/src"
cp backend/Cargo.toml "$BACKUP_DIR/Cargo.toml"

echo "✅ Backup erstellt"
```

---

## 4. Compile-Check nach Extraktion

```bash
#!/bin/bash
# scripts/check-after-extraction.sh

echo "🔍 Prüfe Compilation..."

cd /Users/niklas/Development/30_Development_Code/31_git_ownbusiness/erynoa/erynoa-main/backend

cargo check 2>&1 | tee /tmp/pluto_check.log

if [ $? -eq 0 ]; then
    echo "✅ Compilation erfolgreich"
else
    echo "❌ Compilation fehlgeschlagen - siehe /tmp/pluto_check.log"
    exit 1
fi

echo "🧪 Führe Tests aus..."
cargo test 2>&1 | tee /tmp/pluto_test.log

if [ $? -eq 0 ]; then
    echo "✅ Tests erfolgreich"
else
    echo "⚠️ Einige Tests fehlgeschlagen - siehe /tmp/pluto_test.log"
fi
```

---

## 5. Sed-Patterns für Imports

```bash
#!/bin/bash
# scripts/update-imports.sh
# Aktualisiert Import-Pfade nach Refactoring

# core::state → nervous_system
find backend/src -name "*.rs" -exec sed -i '' \
    's/use crate::core::state::/use crate::nervous_system::/g' {} \;

# peer::p2p → p2p
find backend/src -name "*.rs" -exec sed -i '' \
    's/use crate::peer::p2p::/use crate::p2p::/g' {} \;

# local → storage
find backend/src -name "*.rs" -exec sed -i '' \
    's/use crate::local::/use crate::storage::/g' {} \;

echo "✅ Import-Pfade aktualisiert"
```

---

## 6. Deprecated-Annotations Generator

```rust
// scripts/generate-deprecations.rs
// Generiert deprecated Re-Exports

fn main() {
    let deprecations = vec![
        ("core::state::UnifiedState", "nervous_system::UnifiedState"),
        ("core::state::StateEvent", "nervous_system::event_sourcing::StateEvent"),
        ("peer::p2p::SwarmManager", "p2p::swarm::SwarmManager"),
        ("local::DecentralizedStorage", "storage::DecentralizedStorage"),
    ];

    println!("// core/mod.rs - Deprecated Re-Exports");
    for (old, new) in deprecations {
        println!(
            "#[deprecated(since = \"0.5.0\", note = \"Use {} instead\")]",
            new
        );
        println!("pub use crate::{}::*;", new.split("::").next().unwrap());
    }
}
```

---

## 7. Zeilen-Counter

```bash
#!/bin/bash
# scripts/count-lines.sh
# Zählt Zeilen pro Modul

echo "📊 Zeilen pro Modul:"
echo ""

for dir in nervous_system synapses identity engines realm protection p2p storage eclvm; do
    if [ -d "backend/src/$dir" ]; then
        lines=$(find "backend/src/$dir" -name "*.rs" -exec cat {} \; | wc -l)
        printf "%-20s %8d Zeilen\n" "$dir" "$lines"
    fi
done

echo ""
echo "State.rs spezifisch:"
wc -l backend/src/core/state.rs
```

---

## 8. Rollback-Script

```bash
#!/bin/bash
# scripts/rollback.sh

if [ -z "$1" ]; then
    echo "Usage: ./rollback.sh <backup_timestamp>"
    echo "Available backups:"
    ls -la ../backups/
    exit 1
fi

BACKUP="../backups/pluto_$1"

if [ ! -d "$BACKUP" ]; then
    echo "❌ Backup nicht gefunden: $BACKUP"
    exit 1
fi

echo "⚠️ Rollback zu $BACKUP"
read -p "Fortfahren? (y/n) " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -rf backend/src
    cp -r "$BACKUP/src" backend/src
    cp "$BACKUP/Cargo.toml" backend/Cargo.toml
    echo "✅ Rollback abgeschlossen"
else
    echo "Abgebrochen"
fi
```

---

## 9. Ausführungsreihenfolge

```bash
# 1. Backup erstellen
./scripts/backup-before-refactor.sh

# 2. Struktur erstellen
./scripts/pluto-setup.sh

# 3. Pro Extraktion:
./scripts/extract-event-bus.sh
./scripts/check-after-extraction.sh

# 4. Nach allen Extraktionen:
./scripts/update-imports.sh
./scripts/check-after-extraction.sh

# 5. Zeilen zählen zur Validierung
./scripts/count-lines.sh
```

---

## 10. Git Workflow

```bash
# Feature-Branch erstellen
git checkout -b refactor/projekt-pluto

# Nach jeder Phase committen
git add -A
git commit -m "Phase 1: Foundation - Verzeichnisse und Traits"

# Regelmäßig pushen
git push origin refactor/projekt-pluto

# Nach Abschluss
git checkout main
git merge refactor/projekt-pluto
git tag -a v0.5.0 -m "Projekt Pluto Refactoring"
git push --tags
```
