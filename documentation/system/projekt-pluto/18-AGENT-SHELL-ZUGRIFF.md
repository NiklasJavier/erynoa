# 🐚 Agent Shell-Zugriff: Dynamische Systemkonfiguration auf Peers

> **Teil von:** Projekt Pluto
> **Kategorie:** Agent-Infrastruktur & Sicherheit
> **Status:** Konzept & Spezifikation
> **Letzte Aktualisierung:** Februar 2026

---

## Inhaltsverzeichnis

1. [Vision: Autonome System-Agenten](#1-vision-autonome-system-agenten)
2. [Capability-Modell für Shell-Zugriff](#2-capability-modell-für-shell-zugriff)
3. [Agent-Konfiguration für Shell-Zugriff](#3-agent-konfiguration-für-shell-zugriff)
4. [ECL Policy für Shell-Autorisierung](#4-ecl-policy-für-shell-autorisierung)
5. [Sandbox-Ausführungsmodell](#5-sandbox-ausführungsmodell)
6. [Audit-Trail und Monitoring](#6-audit-trail-und-monitoring)
7. [Trust-Auswirkungen von Shell-Aktionen](#7-trust-auswirkungen-von-shell-aktionen)
8. [CLI für Shell-Capability-Management](#8-cli-für-shell-capability-management)
9. **[AI-Agent Realm-Integration](#9-ai-agent-realm-integration)** ← NEU
10. **[Intent & Saga: Host-Crossing](#10-intent--saga-host-crossing)** ← NEU
11. **[KV-Store Zugriff für Agenten](#11-kv-store-zugriff-für-agenten)** ← NEU
12. **[Shared Computing Power via Saga](#12-shared-computing-power-via-saga)** ← NEU
13. [Axiom-Referenz](#13-axiom-referenz)
14. [Weiterführende Dokumente](#14-weiterführende-dokumente)

---

## 1. Vision: Autonome System-Agenten

### 1.1 Problemstellung

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                         AGENT SHELL ACCESS VISION                            ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Anforderung:                                                               ║
║   - Agenten können auf Linux-Peers Shell-Befehle ausführen                  ║
║   - Dynamische Konfiguration des Host-Systems                               ║
║   - Vollständige Audit-Trails und Trust-basierte Autorisierung              ║
║   - Sandboxing und Ressourcen-Limits                                        ║
║                                                                              ║
║   Use Cases:                                                                 ║
║   - DevOps-Agent verwaltet Container/Services                               ║
║   - Monitoring-Agent reagiert auf Anomalien                                 ║
║   - Update-Agent führt System-Patches durch                                 ║
║   - Backup-Agent orchestriert Datensicherung                                ║
║                                                                              ║
║   Sicherheitsanforderungen:                                                  ║
║   - Capability-basierte Zugriffskontrolle (Κ8)                              ║
║   - Trust-basierte Eskalation (6D Trust-Vektor)                             ║
║   - Sandboxed Execution (keine Root-Escape)                                  ║
║   - Revocation bei Trust-Verlust                                            ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 1.2 Architektur-Übersicht

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          AGENT SHELL ARCHITECTURE                                │
│                                                                                  │
│  ┌─────────────────────┐                     ┌─────────────────────────────────┐│
│  │     AGENT           │                     │         PEER (Linux Host)       ││
│  │ did:erynoa:agent:*  │                     │                                 ││
│  │                     │                     │  ┌───────────────────────────┐  ││
│  │  ┌───────────────┐  │     Erynoa P2P      │  │    Shell Access Layer     │  ││
│  │  │ ShellCapability│  │◄──────────────────►│  │                           │  ││
│  │  │ Credentials    │  │                     │  │  ┌─────────────────────┐  │  ││
│  │  │ Trust Vector  │  │                     │  │  │ Capability Verifier │  │  ││
│  │  └───────────────┘  │                     │  │  └─────────────────────┘  │  ││
│  │                     │                     │  │            │              │  ││
│  │  ┌───────────────┐  │                     │  │            ▼              │  ││
│  │  │ Shell Policy  │  │                     │  │  ┌─────────────────────┐  │  ││
│  │  │ (ECL)         │  │                     │  │  │   ECL Policy Gate   │  │  ││
│  │  └───────────────┘  │                     │  │  └─────────────────────┘  │  ││
│  │                     │                     │  │            │              │  ││
│  └─────────────────────┘                     │  │            ▼              │  ││
│                                              │  │  ┌─────────────────────┐  │  ││
│                                              │  │  │  Sandbox Executor   │  │  ││
│                                              │  │  │  (nsjail/bubblewrap)│  │  ││
│                                              │  │  └─────────────────────┘  │  ││
│                                              │  │            │              │  ││
│                                              │  │            ▼              │  ││
│                                              │  │  ┌─────────────────────┐  │  ││
│                                              │  │  │   Linux Shell/API   │  │  ││
│                                              │  │  └─────────────────────┘  │  ││
│                                              │  │                           │  ││
│                                              │  └───────────────────────────┘  ││
│                                              │                                 ││
│                                              └─────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Capability-Modell für Shell-Zugriff

### 2.1 Shell Capability Definition (Κ8-konform)

```rust
/// Shell-Zugriffs-Capabilities für Agenten
/// Erweitert das bestehende Capability-Enum (identity.rs)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShellCapability {
    /// Vollständiger Shell-Zugriff (GEFÄHRLICH - nur für hochvertrauenswürdige Agenten)
    FullShell {
        /// Erlaubte User-Kontexte (z.B. ["root", "erynoa", "deploy"])
        users: Vec<String>,
        /// Timeout pro Befehl in Sekunden
        timeout_secs: u32,
    },

    /// Eingeschränkter Befehlszugriff (Allowlist)
    RestrictedCommands {
        /// Erlaubte Befehle mit Glob-Pattern (z.B. ["systemctl *", "docker *"])
        allowed_commands: Vec<String>,
        /// Verbotene Argumente (Blocklist)
        blocked_args: Vec<String>,
        /// User-Kontext
        user: String,
        /// Timeout
        timeout_secs: u32,
    },

    /// Pfad-basierter Zugriff
    PathAccess {
        /// Erlaubte Pfade (z.B. ["/etc/erynoa/**", "/var/log/**"])
        paths: Vec<PathPermission>,
        /// User-Kontext
        user: String,
    },

    /// Service-Management
    ServiceControl {
        /// Erlaubte Services (z.B. ["erynoa-peer", "nginx", "postgresql"])
        services: Vec<String>,
        /// Erlaubte Aktionen
        actions: Vec<ServiceAction>,
    },

    /// Container-Orchestrierung
    ContainerControl {
        /// Container-Runtime (docker, podman, containerd)
        runtime: ContainerRuntime,
        /// Erlaubte Container-Namen/IDs (Glob-Pattern)
        containers: Vec<String>,
        /// Erlaubte Aktionen
        actions: Vec<ContainerAction>,
    },

    /// Cron/Timer-Management
    ScheduledTasks {
        /// Namespace für Timer (z.B. "erynoa-agent-*")
        namespace: String,
        /// Maximale Anzahl gleichzeitiger Tasks
        max_concurrent: u32,
    },

    /// Netzwerk-Konfiguration
    NetworkConfig {
        /// Erlaubte Interfaces (z.B. ["eth0", "docker0"])
        interfaces: Vec<String>,
        /// Erlaubte Aktionen
        actions: Vec<NetworkAction>,
    },

    /// Package-Management (apt, dnf, pacman, etc.)
    PackageManagement {
        /// Package-Manager
        manager: PackageManager,
        /// Erlaubte Packages (Allowlist, Glob-Pattern)
        allowed_packages: Vec<String>,
        /// Nur Updates erlaubt (keine neuen Installs)
        update_only: bool,
    },
}

/// Pfad-Berechtigung
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PathPermission {
    /// Pfad-Pattern (Glob)
    pub path: String,
    /// Read-Zugriff
    pub read: bool,
    /// Write-Zugriff
    pub write: bool,
    /// Execute-Zugriff
    pub execute: bool,
    /// Delete-Zugriff
    pub delete: bool,
}

/// Service-Aktionen
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceAction {
    Start,
    Stop,
    Restart,
    Reload,
    Status,
    Enable,
    Disable,
    Logs,
}

/// Container-Aktionen
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContainerAction {
    Start,
    Stop,
    Restart,
    Logs,
    Exec,
    Inspect,
    Stats,
    Pull,
    Remove,
}

/// Netzwerk-Aktionen
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkAction {
    Status,
    Up,
    Down,
    AddRoute,
    RemoveRoute,
    Firewall,
}

/// Container-Runtimes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContainerRuntime {
    Docker,
    Podman,
    Containerd,
    Nerdctl,
}

/// Package-Manager
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageManager {
    Apt,
    Dnf,
    Yum,
    Pacman,
    Zypper,
    Apk,
    Nix,
}
```

### 2.2 Integration in Erynoa Capability-System

```rust
/// Erweiterte Capability-Enum für Shell-Zugriff
/// Fügt ShellAccess zu den bestehenden Capabilities hinzu
impl Capability {
    /// Shell-Zugriff-Capability (neu)
    pub fn shell_access(cap: ShellCapability) -> Self {
        Self::Custom {
            name: "shell".to_string(),
            params: serde_json::to_string(&cap).unwrap_or_default(),
        }
    }

    /// Parse Shell-Capability aus Custom
    pub fn as_shell(&self) -> Option<ShellCapability> {
        match self {
            Self::Custom { name, params } if name == "shell" => {
                serde_json::from_str(params).ok()
            }
            _ => None,
        }
    }

    /// Ist diese Capability eine Shell-Capability?
    pub fn is_shell(&self) -> bool {
        matches!(self, Self::Custom { name, .. } if name == "shell")
    }
}
```

---

## 3. Agent-Konfiguration für Shell-Zugriff

### 3.1 ECL Agent-Definition mit Shell-Capabilities

```yaml
# Agent-Definition: DevOps-Automation-Agent
agent "devops-automation-agent" {
  id:    "did:erynoa:agent:provider:devops-001"
  type:  provider

  # Zugehörigkeit zum Peer-Operator
  owner: @identity("did:erynoa:org:acme-hosting")

  # Credentials für Shell-Zugriff
  credentials: [
    @ref("did:erynoa:credential:shell:devops-operator"),
    @ref("did:erynoa:credential:container:docker-manager"),
    @ref("did:erynoa:credential:service:systemd-admin")
  ]

  # Shell-Capabilities (explizit delegiert vom Owner)
  shell_capabilities: {
    # Service-Management
    services: {
      allowed: ["erynoa-peer", "nginx", "postgresql", "redis"]
      actions: [start, stop, restart, reload, status, logs]
    }

    # Container-Orchestrierung
    containers: {
      runtime: docker
      allowed: ["erynoa-*", "monitoring-*"]
      actions: [start, stop, restart, logs, exec, inspect]
    }

    # Pfad-Zugriff
    paths: [
      { path: "/etc/erynoa/**", read: true, write: true },
      { path: "/var/log/erynoa/**", read: true },
      { path: "/opt/erynoa/**", read: true, write: true, execute: true }
    ]

    # Eingeschränkte Befehle
    commands: {
      allowed: [
        "systemctl status *",
        "docker compose *",
        "journalctl -u erynoa-*",
        "tail -f /var/log/erynoa/*"
      ]
      blocked_args: ["--privileged", "-v /:/host", "rm -rf /"]
    }
  }

  # Trust-Schwellen für Shell-Aktionen
  trust_requirements: {
    service_management: { omega: 0.7, reliability: 0.8 }
    container_control:  { omega: 0.75, competence: 0.7 }
    path_write:         { omega: 0.8, integrity: 0.85 }
    full_shell:         { omega: 0.95, reliability: 0.95, integrity: 0.95 }
  }

  # Policy für automatische Eskalation
  escalation_policy: @ref("did:erynoa:policy:shell-escalation")

  # Aktive Realms
  active_realms: [
    @ref("did:erynoa:realm:acme-infrastructure"),
    @ref("did:erynoa:realm:acme-monitoring")
  ]
}
```

### 3.2 Peer-seitige Shell-Konfiguration

```toml
# /etc/erynoa/shell-access.toml
# Peer-seitige Konfiguration für Shell-Zugriff durch Agenten

[shell_access]
# Aktiviert Shell-Zugriff für Agenten
enabled = true

# Sandbox-Technologie (nsjail, bubblewrap, firejail)
sandbox = "nsjail"

# Default-Timeout für Befehle (Sekunden)
default_timeout_secs = 30

# Maximale parallele Shell-Sessions
max_concurrent_sessions = 10

# Audit-Logging
audit_log_path = "/var/log/erynoa/shell-audit.log"
audit_log_level = "detailed"  # minimal, standard, detailed

# ============================================================================
# Trust-Schwellen (Override der Agent-Konfiguration)
# ============================================================================

[shell_access.trust_thresholds]
# Minimum Trust für jeglichen Shell-Zugriff
global_minimum = { omega = 0.6 }

# Trust für Read-Only-Operationen
read_only = { omega = 0.5, reliability = 0.5 }

# Trust für Service-Control
service_control = { omega = 0.7, reliability = 0.7, competence = 0.6 }

# Trust für Container-Operations
container_control = { omega = 0.75, reliability = 0.7, competence = 0.7 }

# Trust für File-Write-Operations
file_write = { omega = 0.8, integrity = 0.8, reliability = 0.75 }

# Trust für Package-Management
package_management = { omega = 0.85, integrity = 0.85, reliability = 0.8 }

# Trust für Full-Shell (wenn überhaupt erlaubt)
full_shell = { omega = 0.95, integrity = 0.95, reliability = 0.95, competence = 0.9 }

# ============================================================================
# Sandbox-Konfiguration
# ============================================================================

[shell_access.sandbox]
# nsjail Konfiguration
[shell_access.sandbox.nsjail]
# Binary-Pfad
binary = "/usr/bin/nsjail"

# Chroot-Base
chroot_base = "/var/lib/erynoa/sandbox"

# Netzwerk-Isolation
network_namespace = true

# Mount-Points (read-only by default)
mounts = [
    { src = "/bin", dst = "/bin", writable = false },
    { src = "/usr/bin", dst = "/usr/bin", writable = false },
    { src = "/lib", dst = "/lib", writable = false },
    { src = "/lib64", dst = "/lib64", writable = false },
    { src = "/etc/resolv.conf", dst = "/etc/resolv.conf", writable = false },
]

# Dynamische Mounts basierend auf Capabilities
dynamic_mounts = true

# Ressourcen-Limits
[shell_access.sandbox.limits]
max_memory_mb = 512
max_cpu_percent = 50
max_processes = 100
max_open_files = 1024
max_file_size_mb = 100

# ============================================================================
# Allowlist / Blocklist
# ============================================================================

[shell_access.allowlist]
# Global erlaubte Befehle (zusätzlich zu Agent-Capabilities)
commands = [
    "echo",
    "cat",
    "head",
    "tail",
    "grep",
    "awk",
    "sed",
    "ls",
    "pwd",
    "whoami",
    "date",
]

# Erlaubte Pfade für Read
read_paths = [
    "/etc/erynoa/**",
    "/var/log/erynoa/**",
    "/opt/erynoa/**",
]

[shell_access.blocklist]
# Global verbotene Befehle (Override Agent-Capabilities)
commands = [
    "rm -rf /",
    "dd if=/dev/zero",
    ":(){ :|:& };:",  # Fork-Bomb
    "mkfs.*",
    "shutdown",
    "reboot",
    "halt",
    "init 0",
    "init 6",
]

# Verbotene Argumente
arguments = [
    "--privileged",
    "-v /:/host",
    "--pid=host",
    "--net=host",
    "--cap-add=ALL",
]

# Verbotene Pfade
paths = [
    "/etc/passwd",
    "/etc/shadow",
    "/root/**",
    "/home/*/.ssh/**",
    "/boot/**",
]

# ============================================================================
# Agent-spezifische Overrides
# ============================================================================

[[shell_access.agent_overrides]]
# DID des Agenten
agent_did = "did:erynoa:agent:provider:devops-001"

# Erhöhte Limits für diesen Agenten
max_concurrent_sessions = 20
timeout_secs = 120

# Zusätzliche erlaubte Pfade
additional_read_paths = ["/var/lib/docker/**"]
additional_write_paths = ["/etc/nginx/conf.d/**"]

# Zusätzliche erlaubte Befehle
additional_commands = ["nginx -t", "nginx -s reload"]

[[shell_access.agent_overrides]]
agent_did = "did:erynoa:agent:oracle:monitoring-001"
# Nur Read-Only für Monitoring
read_only_mode = true
allowed_commands = ["cat", "grep", "tail", "journalctl"]
```

---

## 4. ECL Policy für Shell-Autorisierung

### 4.1 Shell-Access Gateway Policy

```ecl
// Shell-Access Policy für Peer
// Wird bei jedem Shell-Befehl evaluiert

policy ShellAccessGateway {
    // ============================================
    // Konstanten
    // ============================================
    const MIN_OMEGA = 0.6;
    const MIN_RELIABILITY_SERVICE = 0.7;
    const MIN_INTEGRITY_WRITE = 0.8;
    const FULL_SHELL_OMEGA = 0.95;

    // ============================================
    // 1. Basis-Checks
    // ============================================

    // Agent muss gültige Shell-Credentials haben
    require has_credential(sender, "shell-access"),
        "Shell access credential required";

    // Globaler Minimum-Trust
    require trust.omega >= MIN_OMEGA,
        "Insufficient omega trust for shell access";

    // Agent darf nicht gebannt sein
    require !is_banned(sender),
        "Agent is banned from shell access";

    // ============================================
    // 2. Capability-spezifische Checks
    // ============================================

    // Service-Control
    if action_type == "service_control" {
        require trust.r >= MIN_RELIABILITY_SERVICE,
            "Service control requires reliability >= 0.7";
        require trust.c >= 0.6,
            "Service control requires competence >= 0.6";
        require has_capability(sender, "shell:service_control"),
            "Service control capability not delegated";
    }

    // Container-Control
    if action_type == "container_control" {
        require trust.r >= 0.7 && trust.c >= 0.7,
            "Container control requires R >= 0.7, C >= 0.7";
        require has_capability(sender, "shell:container_control"),
            "Container control capability not delegated";
    }

    // File-Write
    if action_type == "file_write" {
        require trust.i >= MIN_INTEGRITY_WRITE,
            "File write requires integrity >= 0.8";
        require trust.r >= 0.75,
            "File write requires reliability >= 0.75";
        require has_capability(sender, "shell:path_write:" + target_path),
            "Write capability not granted for path";
    }

    // Package-Management
    if action_type == "package_management" {
        require trust.omega >= 0.85,
            "Package management requires omega >= 0.85";
        require trust.i >= 0.85 && trust.r >= 0.8,
            "Package management requires high integrity and reliability";
        require has_capability(sender, "shell:package_management"),
            "Package management capability not delegated";

        // Extra: Nur Updates ohne neue Installs
        if is_new_install && !has_capability(sender, "shell:package_install") {
            return false;
        }
    }

    // Full-Shell (sehr restriktiv)
    if action_type == "full_shell" {
        require trust.omega >= FULL_SHELL_OMEGA,
            "Full shell requires omega >= 0.95";
        require trust.r >= 0.95 && trust.i >= 0.95,
            "Full shell requires R >= 0.95, I >= 0.95";
        require has_capability(sender, "shell:full"),
            "Full shell capability not delegated";

        // Extra: Owner-Bestätigung erforderlich
        if !has_recent_owner_confirmation(sender, hours = 24) {
            escalate("Owner confirmation required for full shell access");
        }
    }

    // ============================================
    // 3. Blocklist-Check
    // ============================================

    require !is_blocked_command(command),
        "Command is blocked by security policy";

    require !contains_blocked_args(command, arguments),
        "Command contains blocked arguments";

    require !is_blocked_path(target_path),
        "Target path is blocked by security policy";

    // ============================================
    // 4. Rate-Limiting
    // ============================================

    let recent_commands = count_recent_commands(sender, minutes = 10);
    let rate_limit = get_rate_limit(sender);

    require recent_commands < rate_limit,
        "Rate limit exceeded for shell commands";

    // ============================================
    // 5. Audit-Log
    // ============================================

    // Implizit: Alle Shell-Commands werden geloggt
    // Audit-Entry wird bei Policy-Pass erstellt

    return true;
}
```

### 4.2 Eskalations-Policy für kritische Aktionen

```ecl
// Eskalations-Policy für Shell-Aktionen
// Definiert wann Owner-Intervention erforderlich ist

policy ShellEscalation {
    const ESCALATE_THRESHOLD = 0.85;

    // ============================================
    // Auto-Reject: Sofortige Ablehnung
    // ============================================

    // Kritische Befehle sofort ablehnen
    if is_critical_command(command) {
        return reject("Critical command requires manual approval");
    }

    // Trust unter Minimum
    if trust.omega < 0.5 {
        return reject("Trust too low for any shell access");
    }

    // ============================================
    // Auto-Accept: Automatische Genehmigung
    // ============================================

    // Routine-Operationen mit hohem Trust
    if is_routine_operation(command) && trust.omega >= 0.8 {
        return accept();
    }

    // Read-Only mit ausreichendem Trust
    if is_read_only(command) && trust.omega >= 0.6 {
        return accept();
    }

    // ============================================
    // Eskalation an Owner
    // ============================================

    // Grenzfälle eskalieren
    if trust.omega >= 0.6 && trust.omega < ESCALATE_THRESHOLD {
        return escalate({
            reason: "Trust in escalation range",
            context: {
                command: command,
                trust: trust,
                agent: sender
            },
            timeout_hours: 24,
            fallback: "reject"
        });
    }

    // Unbekannte Befehle eskalieren
    if !is_known_command(command) {
        return escalate({
            reason: "Unknown command pattern",
            context: { command: command },
            timeout_hours: 4,
            fallback: "reject"
        });
    }

    // Default: Accept wenn alle Checks bestanden
    return accept();
}
```

---

## 5. Sandbox-Ausführungsmodell

### 5.1 Sandbox-Executor Architektur

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          SANDBOX EXECUTOR ARCHITECTURE                           │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────────┐│
│  │                         Shell Request Pipeline                               ││
│  │                                                                              ││
│  │   1. Request ──► 2. Auth ──► 3. Policy ──► 4. Sandbox ──► 5. Execute       ││
│  │       │             │            │             │              │              ││
│  │       ▼             ▼            ▼             ▼              ▼              ││
│  │   Validate     Verify DID    ECL Policy   Setup nsjail   Run Command        ││
│  │   Command      + Caps        Evaluation   Environment    in Sandbox          ││
│  │                                                                              ││
│  └─────────────────────────────────────────────────────────────────────────────┘│
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────────┐│
│  │                         Sandbox Layers                                       ││
│  │                                                                              ││
│  │   ┌─────────────────────────────────────────────────────────────────────┐   ││
│  │   │  Layer 1: Namespace Isolation (nsjail)                               │   ││
│  │   │  - PID namespace (isolierte Prozess-IDs)                             │   ││
│  │   │  - Network namespace (optionale Netzwerk-Isolation)                  │   ││
│  │   │  - Mount namespace (eingeschränkte Filesystem-Sicht)                 │   ││
│  │   │  - User namespace (unprivilegierter User innerhalb)                  │   ││
│  │   │  - IPC namespace (isolierte Inter-Process-Communication)             │   ││
│  │   └─────────────────────────────────────────────────────────────────────┘   ││
│  │                                    │                                         ││
│  │                                    ▼                                         ││
│  │   ┌─────────────────────────────────────────────────────────────────────┐   ││
│  │   │  Layer 2: Seccomp Filtering                                          │   ││
│  │   │  - Nur erlaubte Syscalls                                             │   ││
│  │   │  - Blockiert: ptrace, mount, umount, reboot, ...                     │   ││
│  │   │  - Audit-Logging für blockierte Calls                                │   ││
│  │   └─────────────────────────────────────────────────────────────────────┘   ││
│  │                                    │                                         ││
│  │                                    ▼                                         ││
│  │   ┌─────────────────────────────────────────────────────────────────────┐   ││
│  │   │  Layer 3: Resource Limits (cgroups v2)                               │   ││
│  │   │  - Memory: max 512MB (konfigurierbar)                                │   ││
│  │   │  - CPU: max 50% (konfigurierbar)                                     │   ││
│  │   │  - PIDs: max 100 Prozesse                                            │   ││
│  │   │  - I/O: Bandbreiten-Limit                                            │   ││
│  │   └─────────────────────────────────────────────────────────────────────┘   ││
│  │                                    │                                         ││
│  │                                    ▼                                         ││
│  │   ┌─────────────────────────────────────────────────────────────────────┐   ││
│  │   │  Layer 4: Capability-Based Mounts                                    │   ││
│  │   │  - Read-Only-Bind-Mounts für erlaubte Pfade                          │   ││
│  │   │  - Write-Mounts nur für explizit erlaubte Pfade                      │   ││
│  │   │  - Tmpfs für temporäre Dateien                                       │   ││
│  │   └─────────────────────────────────────────────────────────────────────┘   ││
│  │                                                                              ││
│  └─────────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Rust-Implementation des Sandbox-Executors

```rust
/// Sandbox-Executor für Shell-Befehle
///
/// Führt Befehle in einer isolierten nsjail-Umgebung aus,
/// basierend auf den Capabilities des Agenten.
pub struct SandboxExecutor {
    config: SandboxConfig,
    nsjail_binary: PathBuf,
    chroot_base: PathBuf,
    audit_logger: Arc<AuditLogger>,
}

/// Sandbox-Konfiguration
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Memory-Limit in MB
    pub max_memory_mb: u32,
    /// CPU-Limit in Prozent
    pub max_cpu_percent: u32,
    /// Maximale Prozessanzahl
    pub max_processes: u32,
    /// Maximale offene Files
    pub max_open_files: u32,
    /// Timeout in Sekunden
    pub timeout_secs: u32,
    /// Netzwerk-Isolation aktivieren
    pub network_isolation: bool,
    /// Seccomp-Profil
    pub seccomp_profile: SeccompProfile,
}

/// Seccomp-Profile
#[derive(Debug, Clone)]
pub enum SeccompProfile {
    /// Minimales Profil (sehr restriktiv)
    Minimal,
    /// Standard-Profil (Docker-ähnlich)
    Standard,
    /// Erweitert (für Container-Operations)
    Extended,
    /// Custom-Profil (aus Datei)
    Custom(PathBuf),
}

/// Ergebnis einer Shell-Ausführung
#[derive(Debug, Clone)]
pub struct ShellResult {
    /// Exit-Code
    pub exit_code: i32,
    /// Stdout
    pub stdout: String,
    /// Stderr
    pub stderr: String,
    /// Ausführungszeit in Millisekunden
    pub duration_ms: u64,
    /// Resource-Usage
    pub resource_usage: ResourceUsage,
    /// Audit-ID für Referenz
    pub audit_id: String,
}

/// Resource-Usage während der Ausführung
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// Peak Memory in KB
    pub peak_memory_kb: u64,
    /// CPU-Zeit in Millisekunden
    pub cpu_time_ms: u64,
    /// I/O Reads in Bytes
    pub io_read_bytes: u64,
    /// I/O Writes in Bytes
    pub io_write_bytes: u64,
}

impl SandboxExecutor {
    /// Erstelle neuen Executor
    pub fn new(config: SandboxConfig) -> Result<Self> {
        let nsjail_binary = which::which("nsjail")
            .map_err(|_| ShellError::NsjailNotFound)?;

        Ok(Self {
            config,
            nsjail_binary,
            chroot_base: PathBuf::from("/var/lib/erynoa/sandbox"),
            audit_logger: Arc::new(AuditLogger::new()?),
        })
    }

    /// Führe Befehl in Sandbox aus
    pub async fn execute(
        &self,
        request: &ShellRequest,
        capabilities: &[ShellCapability],
    ) -> Result<ShellResult> {
        // 1. Validiere Befehl gegen Capabilities
        self.validate_command(request, capabilities)?;

        // 2. Erstelle nsjail-Konfiguration
        let nsjail_config = self.build_nsjail_config(request, capabilities)?;

        // 3. Starte Audit-Eintrag
        let audit_id = self.audit_logger.start_entry(request)?;

        // 4. Führe in Sandbox aus
        let start = Instant::now();
        let result = self.run_in_sandbox(&nsjail_config, request).await?;
        let duration_ms = start.elapsed().as_millis() as u64;

        // 5. Komplettiere Audit-Eintrag
        self.audit_logger.complete_entry(
            &audit_id,
            &result,
            duration_ms,
        )?;

        Ok(ShellResult {
            exit_code: result.exit_code,
            stdout: result.stdout,
            stderr: result.stderr,
            duration_ms,
            resource_usage: result.resource_usage,
            audit_id,
        })
    }

    /// Validiere Befehl gegen Capabilities
    fn validate_command(
        &self,
        request: &ShellRequest,
        capabilities: &[ShellCapability],
    ) -> Result<()> {
        // Prüfe ob Befehl von irgendeiner Capability erlaubt wird
        for cap in capabilities {
            if self.capability_allows_command(cap, request)? {
                return Ok(());
            }
        }

        Err(ShellError::CommandNotAllowed {
            command: request.command.clone(),
            reason: "No capability grants this command".into(),
        })
    }

    /// Prüfe ob Capability den Befehl erlaubt
    fn capability_allows_command(
        &self,
        cap: &ShellCapability,
        request: &ShellRequest,
    ) -> Result<bool> {
        match cap {
            ShellCapability::FullShell { users, timeout_secs } => {
                // FullShell erlaubt alles (wenn User passt)
                Ok(users.contains(&request.user) &&
                   request.timeout_secs <= *timeout_secs)
            }

            ShellCapability::RestrictedCommands {
                allowed_commands,
                blocked_args,
                user,
                timeout_secs,
            } => {
                // Prüfe User
                if &request.user != user {
                    return Ok(false);
                }

                // Prüfe Timeout
                if request.timeout_secs > *timeout_secs {
                    return Ok(false);
                }

                // Prüfe gegen Blocklist
                for blocked in blocked_args {
                    if request.arguments.iter().any(|a| a.contains(blocked)) {
                        return Ok(false);
                    }
                }

                // Prüfe gegen Allowlist (Glob-Match)
                let full_command = format!("{} {}", request.command, request.arguments.join(" "));
                for pattern in allowed_commands {
                    if glob_match(pattern, &full_command) {
                        return Ok(true);
                    }
                }

                Ok(false)
            }

            ShellCapability::ServiceControl { services, actions } => {
                // Prüfe ob es ein Service-Befehl ist
                if request.command != "systemctl" {
                    return Ok(false);
                }

                // Parse Action und Service aus Argumenten
                let action = request.arguments.get(0).map(String::as_str);
                let service = request.arguments.get(1).map(String::as_str);

                match (action, service) {
                    (Some(act), Some(svc)) => {
                        let action_ok = actions.iter().any(|a| a.matches_str(act));
                        let service_ok = services.iter().any(|s| glob_match(s, svc));
                        Ok(action_ok && service_ok)
                    }
                    _ => Ok(false),
                }
            }

            ShellCapability::ContainerControl { runtime, containers, actions } => {
                // Prüfe Runtime
                let expected_cmd = runtime.command_name();
                if request.command != expected_cmd {
                    return Ok(false);
                }

                // Parse Container-Befehl
                // z.B. "docker start mycontainer"
                let action = request.arguments.get(0).map(String::as_str);
                let container = request.arguments.get(1).map(String::as_str);

                match (action, container) {
                    (Some(act), Some(ctr)) => {
                        let action_ok = actions.iter().any(|a| a.matches_str(act));
                        let container_ok = containers.iter().any(|c| glob_match(c, ctr));
                        Ok(action_ok && container_ok)
                    }
                    _ => Ok(false),
                }
            }

            ShellCapability::PathAccess { paths, user } => {
                // Nur für Datei-Operationen (cat, read, write, etc.)
                if &request.user != user {
                    return Ok(false);
                }

                // Prüfe ob target_path erlaubt ist
                if let Some(target_path) = &request.target_path {
                    for perm in paths {
                        if glob_match(&perm.path, target_path) {
                            // Prüfe spezifische Permission
                            let op = &request.operation;
                            let allowed = match op.as_str() {
                                "read" => perm.read,
                                "write" => perm.write,
                                "execute" => perm.execute,
                                "delete" => perm.delete,
                                _ => false,
                            };
                            if allowed {
                                return Ok(true);
                            }
                        }
                    }
                }

                Ok(false)
            }

            // Weitere Capabilities...
            _ => Ok(false),
        }
    }

    /// Baue nsjail-Konfiguration
    fn build_nsjail_config(
        &self,
        request: &ShellRequest,
        capabilities: &[ShellCapability],
    ) -> Result<NsjailConfig> {
        let mut config = NsjailConfig::default();

        // Basis-Einstellungen
        config.time_limit = request.timeout_secs;
        config.max_memory = self.config.max_memory_mb * 1024 * 1024;
        config.max_pids = self.config.max_processes;

        // Namespace-Isolation
        config.clone_newnet = self.config.network_isolation;
        config.clone_newpid = true;
        config.clone_newipc = true;
        config.clone_newns = true;
        config.clone_newuts = true;

        // Basis-Mounts (Read-Only)
        config.mounts.push(Mount::ro("/bin", "/bin"));
        config.mounts.push(Mount::ro("/usr/bin", "/usr/bin"));
        config.mounts.push(Mount::ro("/lib", "/lib"));
        config.mounts.push(Mount::ro("/lib64", "/lib64"));
        config.mounts.push(Mount::ro("/etc/resolv.conf", "/etc/resolv.conf"));

        // Tmpfs für /tmp
        config.mounts.push(Mount::tmpfs("/tmp", 100 * 1024 * 1024)); // 100MB

        // Capability-basierte Mounts
        for cap in capabilities {
            if let ShellCapability::PathAccess { paths, .. } = cap {
                for perm in paths {
                    let mount_type = if perm.write {
                        MountType::ReadWrite
                    } else {
                        MountType::ReadOnly
                    };
                    config.mounts.push(Mount::new(&perm.path, &perm.path, mount_type));
                }
            }
        }

        // Seccomp
        config.seccomp_policy = self.config.seccomp_profile.to_nsjail_policy()?;

        Ok(config)
    }

    /// Führe Befehl in nsjail-Sandbox aus
    async fn run_in_sandbox(
        &self,
        nsjail_config: &NsjailConfig,
        request: &ShellRequest,
    ) -> Result<RawShellResult> {
        let config_file = self.write_temp_config(nsjail_config)?;

        let mut cmd = tokio::process::Command::new(&self.nsjail_binary);
        cmd.arg("--config").arg(&config_file);
        cmd.arg("--").arg(&request.command);
        cmd.args(&request.arguments);

        // Umgebungsvariablen setzen
        cmd.env_clear();
        cmd.env("PATH", "/usr/bin:/bin");
        cmd.env("HOME", "/tmp");
        cmd.env("USER", &request.user);

        let output = tokio::time::timeout(
            Duration::from_secs(request.timeout_secs as u64),
            cmd.output(),
        ).await
            .map_err(|_| ShellError::Timeout)?
            .map_err(ShellError::ExecutionFailed)?;

        // Cleanup
        std::fs::remove_file(&config_file).ok();

        Ok(RawShellResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            resource_usage: ResourceUsage::default(), // TODO: Aus cgroups lesen
        })
    }
}
```

---

## 6. Audit-Trail und Monitoring

### 6.1 Audit-Event-Struktur

```rust
/// Audit-Event für Shell-Zugriff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellAuditEvent {
    /// Eindeutige Event-ID
    pub id: String,

    /// Timestamp (ISO 8601)
    pub timestamp: DateTime<Utc>,

    /// Agent-DID
    pub agent_did: String,

    /// Peer-ID
    pub peer_id: String,

    /// Realm-ID (in dem der Agent operiert)
    pub realm_id: String,

    /// Befehl
    pub command: String,

    /// Argumente
    pub arguments: Vec<String>,

    /// Target-Pfad (wenn relevant)
    pub target_path: Option<String>,

    /// User-Kontext
    pub user: String,

    /// Capability die den Zugriff erlaubt hat
    pub authorizing_capability: String,

    /// Trust-Vektor zum Zeitpunkt der Ausführung
    pub trust_at_execution: TrustVector6D,

    /// Ergebnis
    pub result: ShellAuditResult,

    /// Dauer in Millisekunden
    pub duration_ms: u64,

    /// Resource-Usage
    pub resource_usage: ResourceUsage,

    /// Policy-Evaluation-Details
    pub policy_details: PolicyEvaluationDetails,
}

/// Audit-Ergebnis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellAuditResult {
    /// Erfolgreich ausgeführt
    Success {
        exit_code: i32,
        stdout_size: usize,
        stderr_size: usize,
    },

    /// Fehler bei Ausführung
    ExecutionError {
        error: String,
    },

    /// Timeout
    Timeout {
        timeout_secs: u32,
    },

    /// Von Policy abgelehnt
    PolicyDenied {
        reason: String,
        policy_id: String,
    },

    /// Capability nicht vorhanden
    CapabilityDenied {
        required_capability: String,
    },

    /// Trust zu niedrig
    TrustInsufficient {
        required: TrustVector6D,
        actual: TrustVector6D,
    },
}

/// Policy-Evaluation-Details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluationDetails {
    /// Policy-ID
    pub policy_id: String,
    /// Gas verbraucht
    pub gas_used: u64,
    /// Mana verbraucht
    pub mana_used: u64,
    /// Evaluationszeit in Mikrosekunden
    pub evaluation_time_us: u64,
    /// Entscheidung
    pub decision: PolicyDecision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyDecision {
    Accept,
    Reject { reason: String },
    Escalate { to: String, timeout_hours: u32 },
}
```

### 6.2 Monitoring-Integration

```toml
# /etc/erynoa/shell-monitoring.toml
# Monitoring-Konfiguration für Shell-Zugriff

[monitoring]
# Prometheus-Metriken aktivieren
prometheus_enabled = true
prometheus_port = 9090

# Alerting
alerting_enabled = true

[monitoring.metrics]
# Shell-Execution Metriken
shell_executions_total = true
shell_execution_duration_seconds = true
shell_execution_errors_total = true
shell_capability_denials_total = true
shell_policy_denials_total = true
shell_trust_denials_total = true
shell_resource_usage = true

[monitoring.alerts]
# Alert bei >10 Denials pro Minute
[[monitoring.alerts.rules]]
name = "HighDenialRate"
expr = "rate(shell_policy_denials_total[1m]) > 10"
severity = "warning"
annotation = "High shell denial rate detected"

# Alert bei Capability-Missbrauch
[[monitoring.alerts.rules]]
name = "CapabilityAbuse"
expr = "rate(shell_capability_denials_total[5m]) > 20"
severity = "critical"
annotation = "Possible capability abuse detected"

# Alert bei Trust-Einbruch nach Shell-Zugriff
[[monitoring.alerts.rules]]
name = "PostShellTrustDrop"
expr = "trust_drop_after_shell > 0.2"
severity = "warning"
annotation = "Significant trust drop after shell access"
```

---

## 7. Trust-Auswirkungen von Shell-Aktionen

### 7.1 Trust-Update nach Shell-Operationen

```rust
/// Trust-Impact von Shell-Operationen
///
/// Shell-Aktionen haben direkten Einfluss auf den Trust-Vektor des Agenten.
/// Erfolgreiche, nicht-destruktive Aktionen erhöhen Trust.
/// Fehler, Timeouts oder verdächtiges Verhalten reduzieren Trust.
pub struct ShellTrustImpact;

impl ShellTrustImpact {
    /// Berechne Trust-Update nach Shell-Operation
    pub fn calculate(
        action: &ShellAction,
        result: &ShellResult,
        context: &ShellContext,
    ) -> TrustDelta {
        let mut delta = TrustDelta::neutral();

        // Basis-Impact nach Ergebnis
        match result.exit_code {
            0 => {
                // Erfolg: Leichte Erhöhung von R (Reliability)
                delta.r += 0.001;

                // Bei komplexen Aktionen auch C (Competence)
                if action.complexity >= ActionComplexity::Medium {
                    delta.c += 0.001;
                }
            }

            1..=127 => {
                // Normaler Fehler: Leichte Reduktion
                delta.r -= 0.002;
            }

            128..=255 => {
                // Signal-Tod (Timeout, Kill): Stärkere Reduktion
                delta.r -= 0.01;
                delta.p -= 0.005; // Predictability sinkt
            }

            _ => {
                // Unbekannter Exit: Vorsichtige Reduktion
                delta.r -= 0.005;
            }
        }

        // Integrity-Impact bei Write-Operationen
        if action.is_write_operation() {
            if result.exit_code == 0 {
                // Erfolgreicher Write ohne Probleme
                delta.i += 0.002;
            } else {
                // Gescheiterter Write
                delta.i -= 0.005;
            }
        }

        // Vigilance-Impact bei Security-relevanten Aktionen
        if action.is_security_relevant() {
            if result.exit_code == 0 && !result.has_security_warnings() {
                delta.v += 0.003;
            } else if result.has_security_warnings() {
                delta.v -= 0.01;
            }
        }

        // Resource-Abuse Detection
        if result.resource_usage.exceeded_soft_limits() {
            delta.omega -= 0.005;
            delta.p -= 0.005;
        }

        if result.resource_usage.exceeded_hard_limits() {
            delta.omega -= 0.02;
            delta.r -= 0.01;
        }

        // Temporal Decay für häufige Operationen
        let ops_last_hour = context.operations_last_hour;
        if ops_last_hour > 100 {
            // Zu viele Operationen: Leichte Reduktion
            delta = delta.scale(0.8);
        }

        delta
    }
}

/// Trust-Delta für 6D-Vektor
#[derive(Debug, Clone)]
pub struct TrustDelta {
    pub r: f32,     // Reliability
    pub i: f32,     // Integrity
    pub c: f32,     // Competence
    pub p: f32,     // Predictability
    pub v: f32,     // Vigilance
    pub omega: f32, // Omega-Alignment
}

impl TrustDelta {
    pub fn neutral() -> Self {
        Self {
            r: 0.0, i: 0.0, c: 0.0, p: 0.0, v: 0.0, omega: 0.0,
        }
    }

    pub fn scale(&self, factor: f32) -> Self {
        Self {
            r: self.r * factor,
            i: self.i * factor,
            c: self.c * factor,
            p: self.p * factor,
            v: self.v * factor,
            omega: self.omega * factor,
        }
    }
}
```

---

## 8. CLI für Shell-Capability-Management

### 8.1 Agent-Shell-Konfiguration via CLI

```bash
# ============================================================================
# ERYNOA CLI: Shell-Capability-Management
# ============================================================================

# Shell-Capabilities für Agent anzeigen
$ erynoa agent shell-caps show did:erynoa:agent:provider:devops-001

Shell Capabilities for did:erynoa:agent:provider:devops-001:
┌─────────────────────────────────────────────────────────────────────────────┐
│ Capability          │ Details                               │ Trust Req.   │
├─────────────────────┼───────────────────────────────────────┼──────────────┤
│ ServiceControl      │ Services: erynoa-*, nginx, postgres   │ Ω≥0.7 R≥0.7  │
│                     │ Actions: start, stop, restart, status │              │
├─────────────────────┼───────────────────────────────────────┼──────────────┤
│ ContainerControl    │ Runtime: docker                       │ Ω≥0.75 C≥0.7 │
│                     │ Containers: erynoa-*, monitoring-*    │              │
│                     │ Actions: start, stop, logs, exec      │              │
├─────────────────────┼───────────────────────────────────────┼──────────────┤
│ PathAccess          │ /etc/erynoa/** (RW)                   │ Ω≥0.8 I≥0.8  │
│                     │ /var/log/erynoa/** (R)                │              │
│                     │ /opt/erynoa/** (RWX)                  │              │
├─────────────────────┼───────────────────────────────────────┼──────────────┤
│ RestrictedCommands  │ systemctl status *, docker compose *  │ Ω≥0.6        │
│                     │ journalctl -u erynoa-*                │              │
└─────────────────────┴───────────────────────────────────────┴──────────────┘

# Shell-Capability delegieren
$ erynoa agent delegate-shell \
    --from did:erynoa:org:acme-hosting \
    --to did:erynoa:agent:provider:devops-001 \
    --capability service-control \
    --services "erynoa-*,nginx,postgresql" \
    --actions "start,stop,restart,reload,status" \
    --trust-factor 0.8 \
    --valid-until "2026-12-31"

Delegation created:
  ID: del-shell-001
  From: did:erynoa:org:acme-hosting
  To: did:erynoa:agent:provider:devops-001
  Capability: ServiceControl
  Trust Factor: 0.8
  Valid Until: 2026-12-31

# Shell-Capability widerrufen
$ erynoa agent revoke-shell \
    --delegation del-shell-001

Delegation del-shell-001 revoked.

# Shell-Audit-Log anzeigen
$ erynoa peer shell-audit --last 20 --agent did:erynoa:agent:provider:devops-001

Shell Audit Log (last 20 entries):
┌──────────────────────────────────────────────────────────────────────────────────────┐
│ Time                │ Agent        │ Command                   │ Result   │ Duration │
├─────────────────────┼──────────────┼───────────────────────────┼──────────┼──────────┤
│ 2026-02-04 10:15:32 │ devops-001   │ systemctl restart nginx   │ Success  │ 2.3s     │
│ 2026-02-04 10:10:15 │ devops-001   │ docker logs erynoa-peer   │ Success  │ 0.8s     │
│ 2026-02-04 10:05:42 │ devops-001   │ cat /etc/erynoa/config    │ Success  │ 0.1s     │
│ 2026-02-04 10:01:08 │ devops-001   │ rm -rf /tmp/cache/*       │ Denied   │ -        │
│                     │              │ (Policy: blocked path)    │          │          │
└──────────────────────────────────────────────────────────────────────────────────────┘

# Peer-Shell-Konfiguration testen
$ erynoa peer shell-test \
    --agent did:erynoa:agent:provider:devops-001 \
    --command "systemctl status nginx" \
    --dry-run

Shell Test (Dry Run):
  Agent: did:erynoa:agent:provider:devops-001
  Command: systemctl status nginx

  Capability Check: ✓ ServiceControl allows "systemctl status nginx"
  Policy Check: ✓ ShellAccessGateway passed
  Trust Check: ✓ Ω=0.82, R=0.85 (required: Ω≥0.7, R≥0.7)
  Sandbox Config: nsjail, 512MB RAM, 50% CPU, 30s timeout

  Result: WOULD EXECUTE (dry-run mode)
```

---

## 9. AI-Agent Realm-Integration

### 9.1 AI-Agent DID-Struktur

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                         AI-AGENT DID ARCHITECTURE                            ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   AI-Agenten erhalten vollwertige DIDs im `agent:ai:*` Namespace:           ║
║                                                                              ║
║   did:erynoa:agent:ai:{model}:{instance}                                    ║
║                                                                              ║
║   Beispiele:                                                                 ║
║   - did:erynoa:agent:ai:claude:claude-001                                   ║
║   - did:erynoa:agent:ai:gpt4:assistant-prod                                 ║
║   - did:erynoa:agent:ai:llama:local-inference                               ║
║   - did:erynoa:agent:ai:custom:company-internal                             ║
║                                                                              ║
║   AI-Agenten haben:                                                          ║
║   ✓ Eigene DID (Identität)                                                  ║
║   ✓ Eigener 6D Trust-Vektor                                                 ║
║   ✓ Multi-Realm Membership                                                   ║
║   ✓ Delegierte Capabilities                                                  ║
║   ✓ KV-Store Zugriff (per Realm)                                            ║
║   ✓ Compute-Quota (Gas/Mana)                                                ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 9.2 AI-Agent Definition

```rust
/// AI-Agent-Typen
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AIAgentType {
    /// Große Sprachmodelle (Claude, GPT, etc.)
    LLM {
        model_family: String,
        version: String,
        provider: AIProvider,
    },

    /// Vision-Modelle
    VisionModel {
        model_family: String,
        capabilities: Vec<VisionCapability>,
    },

    /// Multimodale Modelle
    Multimodal {
        modalities: Vec<Modality>,
    },

    /// Spezialisierte Agenten (Code, Math, etc.)
    Specialized {
        domain: String,
        specialization: String,
    },

    /// Lokale/selbst-gehostete Modelle
    LocalModel {
        model_path: String,
        runtime: InferenceRuntime,
    },
}

/// AI-Provider
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AIProvider {
    Anthropic,
    OpenAI,
    Google,
    Meta,
    Mistral,
    Local,
    Custom(String),
}

/// AI-Agent Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAgentConfig {
    /// Agent-DID
    pub did: DID,

    /// AI-Typ
    pub agent_type: AIAgentType,

    /// Owner (Mensch oder Organisation)
    pub owner: UniversalId,

    /// Aktive Realm-Memberships
    pub realm_memberships: Vec<RealmMembership>,

    /// Delegierte Capabilities
    pub capabilities: Vec<Capability>,

    /// Trust-Initialisierung
    pub initial_trust: TrustInitConfig,

    /// Compute-Budget
    pub compute_budget: ComputeBudget,

    /// Verhaltens-Constraints
    pub behavioral_constraints: Vec<BehavioralConstraint>,
}

/// Realm-Membership für AI-Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmMembership {
    /// Realm-ID
    pub realm_id: RealmId,

    /// Beitritts-Status
    pub status: MembershipStatus,

    /// Realm-spezifische Capabilities
    pub realm_capabilities: Vec<Capability>,

    /// KV-Store-Zugriff
    pub kv_access: KVAccessConfig,

    /// Beitrittszeitpunkt
    pub joined_at: TemporalCoord,

    /// Ablaufzeit (optional)
    pub expires_at: Option<TemporalCoord>,
}

/// Membership-Status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MembershipStatus {
    /// Beitritt beantragt
    Pending,
    /// Aktiv
    Active,
    /// Suspendiert (temporär)
    Suspended { reason: String, until: Option<TemporalCoord> },
    /// Beendet
    Terminated { reason: String },
}
```

### 9.3 Realm-Beitritts-Policy für AI-Agenten

```ecl
// Policy für AI-Agent Realm-Beitritt
// Realms können definieren, ob und welche AI-Agenten beitreten dürfen

policy AIAgentRealmEntry {
    // ============================================
    // Konstanten
    // ============================================
    const MIN_OWNER_TRUST = 0.7;
    const AI_TRUST_DAMPENING = 0.6;  // AI-Agents starten mit 60% des Owner-Trust

    // ============================================
    // 1. Prüfe ob AI-Agenten erlaubt
    // ============================================

    // Realm-Konfiguration: Sind AI-Agenten grundsätzlich erlaubt?
    require realm.config.allow_ai_agents == true,
        "This realm does not allow AI agents";

    // ============================================
    // 2. Prüfe Owner-Trust
    // ============================================

    let owner_did = get_owner(sender);
    let owner_trust = get_trust(owner_did);

    require owner_trust.omega >= MIN_OWNER_TRUST,
        "Owner trust too low for AI agent realm entry";

    // ============================================
    // 3. Prüfe AI-Agent-Typ
    // ============================================

    let agent_type = get_ai_agent_type(sender);

    // Whitelist-Check
    if realm.config.ai_agent_whitelist.len() > 0 {
        require contains(realm.config.ai_agent_whitelist, agent_type.model_family),
            "AI agent type not whitelisted for this realm";
    }

    // Blacklist-Check
    require !contains(realm.config.ai_agent_blacklist, agent_type.model_family),
        "AI agent type is blacklisted in this realm";

    // ============================================
    // 4. Prüfe Provider-Compliance
    // ============================================

    if realm.config.require_verified_provider {
        require is_verified_provider(agent_type.provider),
            "AI provider not verified";
    }

    // ============================================
    // 5. Capability-Einschränkungen
    // ============================================

    // AI-Agenten bekommen eingeschränkte Capabilities im Realm
    let allowed_caps = realm.config.ai_agent_capabilities;

    for cap in sender.requested_capabilities {
        require contains(allowed_caps, cap),
            "Requested capability not allowed for AI agents in this realm";
    }

    // ============================================
    // 6. Compute-Budget prüfen
    // ============================================

    require sender.compute_budget.gas_limit <= realm.config.max_ai_gas_per_hour,
        "AI agent gas budget exceeds realm limit";

    require sender.compute_budget.mana_limit <= realm.config.max_ai_mana_per_hour,
        "AI agent mana budget exceeds realm limit";

    // ============================================
    // 7. Initial Trust berechnen
    // ============================================

    // AI-Agent Trust = Owner-Trust × Dampening-Faktor
    let initial_trust = owner_trust * AI_TRUST_DAMPENING;

    // Setze Trust für den neuen Member
    set_initial_trust(sender, initial_trust);

    return true;
}
```

### 9.4 Multi-Realm AI-Agent Konfiguration (YAML)

```yaml
# AI-Agent Definition mit Multi-Realm Support
agent "claude-research-assistant" {
  id:    "did:erynoa:agent:ai:claude:research-001"
  type:  ai

  # AI-Modell-Details
  ai_config: {
    model_family: "claude"
    model_version: "3.5-sonnet"
    provider: anthropic
    inference_mode: api  # api, local, hybrid
  }

  # Owner (verantwortliche Person/Org)
  owner: @identity("did:erynoa:org:acme-research")

  # Multi-Realm Memberships
  realm_memberships: [
    # Research-Realm: Voller Zugriff
    {
      realm: @ref("did:erynoa:realm:acme-research")
      status: active
      capabilities: [
        "read:*",
        "write:research-notes/*",
        "execute:analysis/*",
        "kv:read:shared/*",
        "kv:write:personal/*"
      ]
      kv_access: {
        shared_read: ["research-data", "publications", "references"]
        shared_write: []
        personal_read: ["*"]
        personal_write: ["notes", "drafts", "analysis"]
      }
      compute_budget: {
        gas_per_hour: 100000
        mana_per_hour: 50000
      }
    },

    # Public-Knowledge-Realm: Nur Lesen
    {
      realm: @ref("did:erynoa:realm:public-knowledge")
      status: active
      capabilities: [
        "read:*",
        "kv:read:shared/*"
      ]
      kv_access: {
        shared_read: ["*"]
        shared_write: []
        personal_read: []
        personal_write: []
      }
      compute_budget: {
        gas_per_hour: 10000
        mana_per_hour: 5000
      }
    },

    # Infrastructure-Realm: Bedingt (Pending Approval)
    {
      realm: @ref("did:erynoa:realm:acme-infrastructure")
      status: pending
      requested_capabilities: [
        "read:monitoring/*",
        "execute:alerts/*",
        "shell:service_control"
      ]
    }
  ]

  # Globale Verhaltens-Constraints
  behavioral_constraints: [
    {
      type: rate_limit
      max_actions_per_minute: 60
    },
    {
      type: no_self_modification
      description: "Agent darf eigene Konfiguration nicht ändern"
    },
    {
      type: human_escalation
      trigger: "high_risk_actions"
      escalation_target: @identity("did:erynoa:person:alice")
    }
  ]

  # Trust-Initialisierung
  initial_trust: {
    inherit_from_owner: true
    dampening_factor: 0.6
    dimensions: {
      r: 0.0   # Reliability startet bei 0 (muss aufgebaut werden)
      i: 0.5   # Integrity erbt teilweise vom Owner
      c: 0.7   # Competence kann hoch starten (AI ist kompetent)
      p: 0.3   # Predictability muss bewiesen werden
      v: 0.0   # Vigilance startet bei 0
      omega: 0.4  # Omega = gewichteter Durchschnitt
    }
  }
}
```

---

## 10. Intent & Saga: Host-Crossing

### 10.1 Erweitertes Crossing-Modell

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                         EXTENDED CROSSING MODEL                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Bisheriges Modell (Κ23):                                                  ║
║   ┌──────────┐                        ┌──────────┐                          ║
║   │  Realm A │  ───── Gateway ─────▶  │  Realm B │                          ║
║   └──────────┘                        └──────────┘                          ║
║                                                                              ║
║   Erweitertes Modell (Κ23+):                                                ║
║                                                                              ║
║   ┌──────────┐                        ┌──────────────────────┐              ║
║   │  Realm   │  ───── Gateway ─────▶  │  HOST (Linux Peer)   │              ║
║   └──────────┘         │              │  ┌────────────────┐  │              ║
║        │               │              │  │  Shell Access  │  │              ║
║        │               │              │  │  File System   │  │              ║
║        │               │              │  │  Containers    │  │              ║
║        │               │              │  │  Services      │  │              ║
║        │               │              │  │  Compute       │  │              ║
║        │               │              │  └────────────────┘  │              ║
║        │               │              └──────────────────────┘              ║
║        │               │                                                     ║
║        │               ▼                                                     ║
║        │        ┌─────────────────────────────────────────┐                 ║
║        │        │  HOST-CROSSING GATEWAY                   │                 ║
║        │        │  • Trust-Check (höhere Schwelle)         │                 ║
║        │        │  • Capability-Verifikation               │                 ║
║        │        │  • Sandbox-Konfiguration                 │                 ║
║        │        │  • Audit-Trail                           │                 ║
║        │        └─────────────────────────────────────────┘                 ║
║        │                                                                     ║
║        ▼                                                                     ║
║   ┌──────────────────────────────────────────────────────────────────────┐  ║
║   │                           SAGA ORCHESTRATION                          │  ║
║   │                                                                        │  ║
║   │  Intent: "Restart nginx after config update"                          │  ║
║   │                                                                        │  ║
║   │  ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐               │  ║
║   │  │ Step 1  │──▶│ Step 2  │──▶│ Step 3  │──▶│ Step 4  │               │  ║
║   │  │ Read    │   │ Validate │   │ Write  │   │ Restart │               │  ║
║   │  │ Config  │   │ Syntax   │   │ Config │   │ nginx   │               │  ║
║   │  │ (Realm) │   │ (Host)   │   │ (Host) │   │ (Host)  │               │  ║
║   │  └─────────┘   └─────────┘   └─────────┘   └─────────┘               │  ║
║   │       │             │             │             │                      │  ║
║   │       │             │             │             │                      │  ║
║   │       ▼             ▼             ▼             ▼                      │  ║
║   │  Compensation: Rollback Config, Restart with old Config                │  ║
║   └──────────────────────────────────────────────────────────────────────┘  ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 10.2 HostCrossing Goal-Type

```rust
/// Erweiterte Goal-Enum für Host-Crossing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Goal {
    // ... bestehende Goals ...

    /// NEU: Host-Crossing für System-Operationen
    HostOperation {
        /// Ziel-Peer
        target_peer: PeerId,
        /// Operation-Typ
        operation: HostOperationType,
        /// Parameter
        params: HashMap<String, serde_json::Value>,
        /// Erforderliche Capabilities
        required_capabilities: Vec<ShellCapability>,
    },

    /// NEU: Multi-Host Orchestration
    DistributedOperation {
        /// Ziel-Peers
        target_peers: Vec<PeerId>,
        /// Operation pro Peer
        operation: HostOperationType,
        /// Ausführungs-Strategie
        strategy: DistributedStrategy,
        /// Aggregation der Ergebnisse
        aggregation: ResultAggregation,
    },

    /// NEU: Compute-Request (Shared Computing)
    ComputeRequest {
        /// Compute-Task-Typ
        task: ComputeTask,
        /// Resource-Anforderungen
        requirements: ComputeRequirements,
        /// Budget
        max_cost: Budget,
    },
}

/// Host-Operation-Typen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostOperationType {
    /// Shell-Befehl ausführen
    ShellExecute {
        command: String,
        args: Vec<String>,
        timeout_secs: u32,
    },

    /// Datei lesen
    FileRead {
        path: String,
    },

    /// Datei schreiben
    FileWrite {
        path: String,
        content: Vec<u8>,
        mode: FileMode,
    },

    /// Service steuern
    ServiceControl {
        service: String,
        action: ServiceAction,
    },

    /// Container steuern
    ContainerControl {
        container: String,
        action: ContainerAction,
    },

    /// Compute-Task ausführen
    ComputeExecute {
        task: ComputeTask,
    },
}

/// Distributed Execution Strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributedStrategy {
    /// Parallel auf allen Peers
    Parallel,
    /// Sequentiell mit Abhängigkeiten
    Sequential,
    /// Rolling Update (X% gleichzeitig)
    Rolling { batch_percent: u8 },
    /// Nur auf einem zufälligen Peer
    AnyOne,
    /// Auf erstem verfügbaren Peer
    FirstAvailable,
}
```

### 10.3 Host-Crossing Saga-Composition

```rust
impl SagaComposer {
    /// Komponiere Host-Operation Saga
    pub fn compose_host_operation(
        &self,
        source: &UniversalId,
        target_peer: &PeerId,
        operation: &HostOperationType,
        required_capabilities: &[ShellCapability],
    ) -> CompositionResult<Vec<SagaStep>> {
        let mut steps = Vec::new();
        let mut step_index = 0;

        // Step 1: Host-Crossing Gateway Check
        steps.push(SagaStep::new(
            step_index,
            format!("Host-Crossing Gateway for Peer {}", target_peer),
            SagaAction::HostGatewayCheck {
                subject: source.clone(),
                target_peer: target_peer.clone(),
                required_trust: 0.7, // Höhere Schwelle für Host-Zugriff
                required_capabilities: required_capabilities.to_vec(),
            },
        ).with_compensation(SagaCompensation::new(
            "Revoke temporary host access",
            SagaAction::RevokeHostAccess {
                subject: source.clone(),
                peer: target_peer.clone(),
            },
        )));
        step_index += 1;

        // Step 2: Sandbox-Vorbereitung
        steps.push(SagaStep::new(
            step_index,
            "Prepare sandbox environment",
            SagaAction::PrepareSandbox {
                peer: target_peer.clone(),
                capabilities: required_capabilities.to_vec(),
                resource_limits: self.config.default_resource_limits.clone(),
            },
        ).with_dependencies(vec![step_index - 1]));
        step_index += 1;

        // Step 3: Operation ausführen
        let operation_step = match operation {
            HostOperationType::ShellExecute { command, args, timeout_secs } => {
                SagaStep::new(
                    step_index,
                    format!("Execute: {} {}", command, args.join(" ")),
                    SagaAction::HostShellExecute {
                        peer: target_peer.clone(),
                        command: command.clone(),
                        args: args.clone(),
                        timeout_secs: *timeout_secs,
                    },
                )
            }
            HostOperationType::ServiceControl { service, action } => {
                SagaStep::new(
                    step_index,
                    format!("Service {} {}", action.as_str(), service),
                    SagaAction::HostServiceControl {
                        peer: target_peer.clone(),
                        service: service.clone(),
                        action: action.clone(),
                    },
                ).with_compensation(SagaCompensation::new(
                    format!("Rollback service {} state", service),
                    SagaAction::HostServiceControl {
                        peer: target_peer.clone(),
                        service: service.clone(),
                        action: action.compensating_action(),
                    },
                ))
            }
            HostOperationType::FileWrite { path, content, mode } => {
                // Backup vor Write
                steps.push(SagaStep::new(
                    step_index,
                    format!("Backup file {}", path),
                    SagaAction::HostFileBackup {
                        peer: target_peer.clone(),
                        path: path.clone(),
                    },
                ).with_dependencies(vec![step_index - 1]));
                step_index += 1;

                SagaStep::new(
                    step_index,
                    format!("Write file {}", path),
                    SagaAction::HostFileWrite {
                        peer: target_peer.clone(),
                        path: path.clone(),
                        content: content.clone(),
                        mode: mode.clone(),
                    },
                ).with_compensation(SagaCompensation::new(
                    format!("Restore backup of {}", path),
                    SagaAction::HostFileRestore {
                        peer: target_peer.clone(),
                        path: path.clone(),
                    },
                ))
            }
            _ => {
                return Err(CompositionError::UnsupportedGoal(
                    format!("{:?}", operation)
                ));
            }
        };

        steps.push(operation_step.with_dependencies(vec![step_index - 1]));
        step_index += 1;

        // Step 4: Sandbox aufräumen
        steps.push(SagaStep::new(
            step_index,
            "Cleanup sandbox",
            SagaAction::CleanupSandbox {
                peer: target_peer.clone(),
            },
        ).with_dependencies(vec![step_index - 1]));

        Ok(steps)
    }
}
```

### 10.4 Host-Crossing Intent Beispiele

```yaml
# Intent: Config-Update mit Service-Restart
intent "update-nginx-config" {
  source: @identity("did:erynoa:agent:ai:claude:devops-001")

  goal: {
    type: host_operation
    target_peer: "peer:acme-webserver-01"
    operation: {
      type: shell_execute
      command: "/opt/erynoa/scripts/update-nginx.sh"
      args: ["--config", "/etc/nginx/sites-enabled/app.conf"]
      timeout_secs: 60
    }
    required_capabilities: [
      { type: path_access, paths: ["/etc/nginx/**"], write: true },
      { type: service_control, services: ["nginx"], actions: [reload] }
    ]
  }

  constraints: [
    { type: max_cost, gas: 10000, mana: 5000 },
    { type: timeout, seconds: 120 }
  ]

  context_realm: @ref("did:erynoa:realm:acme-infrastructure")
}

# Intent: Distributed Log Collection
intent "collect-logs-all-peers" {
  source: @identity("did:erynoa:agent:ai:claude:monitoring-001")

  goal: {
    type: distributed_operation
    target_peers: ["peer:web-*", "peer:api-*"]
    operation: {
      type: shell_execute
      command: "tail"
      args: ["-n", "1000", "/var/log/erynoa/app.log"]
      timeout_secs: 30
    }
    strategy: parallel
    aggregation: collect_all
  }

  constraints: [
    { type: max_peers, count: 50 },
    { type: max_cost, gas: 50000 }
  ]
}
```

---

## 11. KV-Store Zugriff für Agenten

### 11.1 KV-Store Capability-Modell

```rust
/// KV-Store Zugriffs-Capabilities für Agenten
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KVAccessCapability {
    /// Realm-ID (KV-Stores sind realm-gebunden)
    pub realm_id: RealmId,

    /// Store-Pattern (Glob)
    pub store_pattern: String,

    /// Key-Pattern (Glob)
    pub key_pattern: String,

    /// Erlaubte Operationen
    pub operations: Vec<KVOperation>,

    /// Nur persönliche Daten (eigener Namespace)
    pub personal_only: bool,

    /// Rate-Limit (Ops pro Minute)
    pub rate_limit: u32,
}

/// KV-Operationen
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KVOperation {
    /// Lesen
    Read,
    /// Schreiben
    Write,
    /// Löschen
    Delete,
    /// Listen (Keys auflisten)
    List,
    /// Watch (Änderungen beobachten)
    Watch,
    /// Schema lesen
    ReadSchema,
    /// Schema ändern (gefährlich!)
    WriteSchema,
}

/// KV-Access Kontext für einen Agenten
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentKVContext {
    /// Agent-DID
    pub agent_did: DID,

    /// Aktives Realm
    pub realm_id: RealmId,

    /// Verfügbare KV-Capabilities
    pub capabilities: Vec<KVAccessCapability>,

    /// Aktueller Mana-Stand
    pub mana_available: u64,

    /// Rate-Limit Tracker
    pub rate_tracker: RateLimitTracker,
}
```

### 11.2 KV-Store Integration in Agent-Config

```yaml
# Erweiterte Agent-Definition mit KV-Zugriff
agent "data-analyst-ai" {
  id:    "did:erynoa:agent:ai:claude:analyst-001"
  type:  ai

  owner: @identity("did:erynoa:org:acme-analytics")

  # Realm-Memberships mit KV-Zugriff
  realm_memberships: [
    {
      realm: @ref("did:erynoa:realm:acme-analytics")
      status: active

      # Detaillierter KV-Zugriff
      kv_access: {
        # Shared Stores (Realm-weit)
        shared: [
          {
            store: "customer-data"
            key_pattern: "*"
            operations: [read, list]
            rate_limit: 100  # Ops/min
          },
          {
            store: "analytics-results"
            key_pattern: "agent:analyst-001:*"
            operations: [read, write, delete]
            rate_limit: 200
          },
          {
            store: "ml-models"
            key_pattern: "*"
            operations: [read]
            rate_limit: 10
          }
        ]

        # Persönliche Stores (nur für diesen Agent)
        personal: [
          {
            store: "working-memory"
            key_pattern: "*"
            operations: [read, write, delete, list]
            rate_limit: 500
            max_size_mb: 100
          },
          {
            store: "analysis-cache"
            key_pattern: "*"
            operations: [read, write, delete]
            ttl_hours: 24
          }
        ]
      }
    }
  ]

  # KV-Schema-Definitionen die dieser Agent erstellen darf
  allowed_schemas: [
    {
      store_prefix: "agent:analyst-001:"
      max_fields: 20
      max_depth: 3
      allowed_types: [string, number, bool, list, object]
    }
  ]
}
```

### 11.3 KV-Access ECL Policy

```ecl
// KV-Store Access Policy für Agenten
policy AgentKVAccess {
    // ============================================
    // Konstanten
    // ============================================
    const MIN_TRUST_READ = 0.4;
    const MIN_TRUST_WRITE = 0.6;
    const MIN_TRUST_DELETE = 0.7;
    const MIN_TRUST_SCHEMA = 0.85;

    // ============================================
    // 1. Basis-Checks
    // ============================================

    // Agent muss Realm-Member sein
    require is_realm_member(sender, target_realm),
        "Agent is not a member of this realm";

    // Agent muss aktiv sein (nicht suspended)
    require get_membership_status(sender, target_realm) == "active",
        "Agent membership is not active";

    // ============================================
    // 2. Operation-spezifische Trust-Checks
    // ============================================

    if operation == "read" || operation == "list" {
        require trust.omega >= MIN_TRUST_READ,
            "Insufficient trust for read operation";
    }

    if operation == "write" {
        require trust.omega >= MIN_TRUST_WRITE,
            "Insufficient trust for write operation";
        require trust.i >= 0.5,
            "Insufficient integrity for write operation";
    }

    if operation == "delete" {
        require trust.omega >= MIN_TRUST_DELETE,
            "Insufficient trust for delete operation";
        require trust.i >= 0.6 && trust.r >= 0.6,
            "Insufficient integrity/reliability for delete";
    }

    if operation == "write_schema" {
        require trust.omega >= MIN_TRUST_SCHEMA,
            "Insufficient trust for schema modification";
        require has_capability(sender, "kv:schema_write"),
            "Schema write capability not delegated";
    }

    // ============================================
    // 3. Store & Key Pattern Check
    // ============================================

    let kv_caps = get_kv_capabilities(sender, target_realm);

    require any(kv_caps, cap =>
        matches(target_store, cap.store_pattern) &&
        matches(target_key, cap.key_pattern) &&
        contains(cap.operations, operation)
    ), "No capability grants this KV operation";

    // ============================================
    // 4. Personal-Only Check
    // ============================================

    let matching_cap = find(kv_caps, cap =>
        matches(target_store, cap.store_pattern)
    );

    if matching_cap.personal_only {
        require starts_with(target_key, sender.did + ":"),
            "Personal-only capability but accessing non-personal key";
    }

    // ============================================
    // 5. Rate-Limiting
    // ============================================

    let rate_limit = matching_cap.rate_limit;
    let recent_ops = count_kv_ops(sender, target_realm, minutes = 1);

    require recent_ops < rate_limit,
        "KV operation rate limit exceeded";

    // ============================================
    // 6. Mana-Check
    // ============================================

    let op_cost = calculate_kv_cost(operation, target_store, target_key);
    require sender.mana_available >= op_cost,
        "Insufficient mana for KV operation";

    // Deduct mana
    deduct_mana(sender, op_cost);

    return true;
}
```

### 11.4 KV-Store API für Agenten

```rust
/// KV-Store API für Agent-Zugriff
pub trait AgentKVStore: Send + Sync {
    /// Lese Wert
    async fn get(
        &self,
        ctx: &AgentKVContext,
        store: &str,
        key: &str,
    ) -> Result<Option<StoreValue>>;

    /// Schreibe Wert
    async fn set(
        &self,
        ctx: &AgentKVContext,
        store: &str,
        key: &str,
        value: StoreValue,
    ) -> Result<()>;

    /// Lösche Wert
    async fn delete(
        &self,
        ctx: &AgentKVContext,
        store: &str,
        key: &str,
    ) -> Result<bool>;

    /// Liste Keys
    async fn list(
        &self,
        ctx: &AgentKVContext,
        store: &str,
        prefix: &str,
        limit: usize,
    ) -> Result<Vec<String>>;

    /// Watch für Änderungen
    async fn watch(
        &self,
        ctx: &AgentKVContext,
        store: &str,
        key_pattern: &str,
    ) -> Result<KVWatchStream>;

    /// Batch-Operationen
    async fn batch(
        &self,
        ctx: &AgentKVContext,
        operations: Vec<KVBatchOperation>,
    ) -> Result<Vec<KVBatchResult>>;
}

/// Intent-basierter KV-Zugriff via Saga
impl SagaComposer {
    pub fn compose_kv_operation(
        &self,
        source: &UniversalId,
        realm: &RealmId,
        kv_ops: Vec<KVOperation>,
    ) -> CompositionResult<Vec<SagaStep>> {
        let mut steps = Vec::new();

        // Step 1: KV-Gateway Check
        steps.push(SagaStep::new(
            0,
            "KV Access Gateway Check",
            SagaAction::KVGatewayCheck {
                subject: source.clone(),
                realm: realm.clone(),
                operations: kv_ops.iter().map(|op| op.operation_type()).collect(),
            },
        ));

        // Step 2-N: KV-Operationen
        for (i, op) in kv_ops.iter().enumerate() {
            let step = SagaStep::new(
                i + 1,
                format!("KV: {} {}/{}", op.operation_type(), op.store, op.key),
                SagaAction::KVOperation {
                    realm: realm.clone(),
                    operation: op.clone(),
                },
            ).with_dependencies(vec![i]);

            // Compensation für Write/Delete
            if matches!(op.operation_type(), "write" | "delete") {
                steps.push(step.with_compensation(SagaCompensation::new(
                    format!("Rollback KV {}/{}", op.store, op.key),
                    SagaAction::KVRollback {
                        realm: realm.clone(),
                        store: op.store.clone(),
                        key: op.key.clone(),
                    },
                )));
            } else {
                steps.push(step);
            }
        }

        Ok(steps)
    }
}
```

---

## 12. Shared Computing Power via Saga

### 12.1 Distributed Compute Architektur

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    SHARED COMPUTING POWER ARCHITECTURE                        ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   ┌────────────────────────────────────────────────────────────────────────┐║
║   │                        COMPUTE MARKETPLACE                              │║
║   │                                                                         │║
║   │   Requester (Agent)              Provider (Peer)                        │║
║   │   ┌───────────────┐             ┌───────────────┐                       │║
║   │   │ ComputeIntent │             │ ComputeOffer  │                       │║
║   │   │               │◄──────────►│               │                        │║
║   │   │ • Task-Type   │  Matching   │ • Capacity    │                       │║
║   │   │ • Resources   │             │ • Price       │                       │║
║   │   │ • Budget      │             │ • Trust       │                       │║
║   │   │ • Deadline    │             │ • Specs       │                       │║
║   │   └───────────────┘             └───────────────┘                       │║
║   │                                                                         │║
║   └────────────────────────────────────────────────────────────────────────┘║
║                                       │                                      ║
║                                       ▼                                      ║
║   ┌────────────────────────────────────────────────────────────────────────┐║
║   │                         SAGA ORCHESTRATION                              │║
║   │                                                                         │║
║   │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐      │║
║   │  │ Match   │─▶│ Lock    │─▶│ Transfer│─▶│ Execute │─▶│ Verify  │      │║
║   │  │ Offer   │  │ Payment │  │ Task    │  │ Compute │  │ Result  │      │║
║   │  └─────────┘  └─────────┘  └─────────┘  └─────────┘  └─────────┘      │║
║   │       │            │            │            │            │            │║
║   │       └────────────┴────────────┴────────────┴────────────┘            │║
║   │                           COMPENSATION                                  │║
║   │                      (Refund on Failure)                                │║
║   │                                                                         │║
║   └────────────────────────────────────────────────────────────────────────┘║
║                                       │                                      ║
║                                       ▼                                      ║
║   ┌────────────────────────────────────────────────────────────────────────┐║
║   │                      EXECUTION ENVIRONMENTS                             │║
║   │                                                                         │║
║   │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │║
║   │  │   WASM       │  │  Container   │  │   Native     │                  │║
║   │  │  Sandbox     │  │  (Docker)    │  │  (Trusted)   │                  │║
║   │  │              │  │              │  │              │                  │║
║   │  │ • Secure     │  │ • Flexible   │  │ • Fast       │                  │║
║   │  │ • Portable   │  │ • Isolated   │  │ • Full Access│                  │║
║   │  │ • Metered    │  │ • Standard   │  │ • High Trust │                  │║
║   │  └──────────────┘  └──────────────┘  └──────────────┘                  │║
║   │                                                                         │║
║   └────────────────────────────────────────────────────────────────────────┘║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 12.2 Compute Task Types

```rust
/// Compute-Task Typen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeTask {
    /// WASM-Modul ausführen
    WasmModule {
        /// WASM-Bytecode (oder CID für CAS-Lookup)
        module: WasmSource,
        /// Eingangsparameter
        input: Vec<u8>,
        /// Erwartete Ausführungszeit (ms)
        expected_duration_ms: u64,
    },

    /// Container-basierte Ausführung
    Container {
        /// Container-Image (verifiziert)
        image: ContainerImage,
        /// Umgebungsvariablen
        env: HashMap<String, String>,
        /// Eingabedaten
        input_data: DataSource,
        /// Ausgabe-Pfad
        output_path: String,
    },

    /// ML-Inferenz
    MLInference {
        /// Modell-Referenz
        model: ModelReference,
        /// Input-Daten
        input: InferenceInput,
        /// Batch-Größe
        batch_size: u32,
    },

    /// Map-Reduce über Daten
    MapReduce {
        /// Map-Funktion (WASM)
        mapper: WasmSource,
        /// Reduce-Funktion (WASM)
        reducer: WasmSource,
        /// Eingabedaten (CIDs)
        input_chunks: Vec<ContentId>,
    },

    /// Shell-Script (nur für trusted Peers)
    Script {
        /// Script-Inhalt
        script: String,
        /// Interpreter
        interpreter: String,
        /// Erforderlicher Trust
        required_trust: f64,
    },
}

/// Resource-Anforderungen für Compute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeRequirements {
    /// Minimale CPU-Kerne
    pub min_cpu_cores: u32,
    /// Minimaler RAM (MB)
    pub min_memory_mb: u64,
    /// Minimaler Speicher (MB)
    pub min_storage_mb: u64,
    /// GPU erforderlich?
    pub gpu_required: bool,
    /// GPU-Typ (wenn erforderlich)
    pub gpu_type: Option<GpuType>,
    /// Maximale Latenz (ms)
    pub max_latency_ms: u32,
    /// Geografische Einschränkungen
    pub geo_constraints: Vec<GeoConstraint>,
}

/// Compute-Angebot von einem Peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeOffer {
    /// Anbietender Peer
    pub provider: PeerId,
    /// Provider-DID
    pub provider_did: DID,
    /// Verfügbare Kapazität
    pub capacity: ComputeCapacity,
    /// Preis pro Compute-Unit
    pub price_per_unit: Price,
    /// Provider-Trust
    pub trust: TrustVector6D,
    /// Unterstützte Task-Typen
    pub supported_tasks: Vec<ComputeTaskType>,
    /// Gültigkeitsdauer des Angebots
    pub valid_until: TemporalCoord,
}
```

### 12.3 Compute-Saga Composition

```rust
impl SagaComposer {
    /// Komponiere Distributed Compute Saga
    pub fn compose_compute_request(
        &self,
        requester: &UniversalId,
        task: &ComputeTask,
        requirements: &ComputeRequirements,
        budget: &Budget,
    ) -> CompositionResult<Vec<SagaStep>> {
        let mut steps = Vec::new();
        let mut step_idx = 0;

        // Step 1: Match Compute Offers
        steps.push(SagaStep::new(
            step_idx,
            "Find matching compute providers",
            SagaAction::ComputeMatch {
                requirements: requirements.clone(),
                task_type: task.task_type(),
                max_providers: 10,
            },
        ));
        step_idx += 1;

        // Step 2: Select Best Provider (Trust × Price × Capacity)
        steps.push(SagaStep::new(
            step_idx,
            "Select optimal provider",
            SagaAction::ComputeSelect {
                selection_strategy: ComputeSelectionStrategy::TrustWeighted {
                    trust_weight: 0.4,
                    price_weight: 0.3,
                    capacity_weight: 0.3,
                },
            },
        ).with_dependencies(vec![step_idx - 1]));
        step_idx += 1;

        // Step 3: Lock Payment
        steps.push(SagaStep::new(
            step_idx,
            "Lock payment for compute",
            SagaAction::Lock {
                owner: requester.clone(),
                amount: budget.max_cost,
                asset_type: budget.currency.clone(),
            },
        ).with_dependencies(vec![step_idx - 1])
         .with_compensation(SagaCompensation::new(
            "Unlock payment on failure",
            SagaAction::Unlock {
                lock_id: "$lock_id".to_string(),
            },
        )));
        step_idx += 1;

        // Step 4: Transfer Task to Provider
        steps.push(SagaStep::new(
            step_idx,
            "Transfer compute task to provider",
            SagaAction::ComputeTaskTransfer {
                task: task.clone(),
                to_provider: "$selected_provider".to_string(),
            },
        ).with_dependencies(vec![step_idx - 1]));
        step_idx += 1;

        // Step 5: Execute Task (async, with progress updates)
        steps.push(SagaStep::new(
            step_idx,
            "Execute compute task",
            SagaAction::ComputeExecute {
                task_id: "$task_id".to_string(),
                timeout_ms: requirements.max_latency_ms as u64 * 1000,
                progress_callback: true,
            },
        ).with_dependencies(vec![step_idx - 1])
         .with_compensation(SagaCompensation::new(
            "Abort compute task",
            SagaAction::ComputeAbort {
                task_id: "$task_id".to_string(),
            },
        )));
        step_idx += 1;

        // Step 6: Verify Result
        steps.push(SagaStep::new(
            step_idx,
            "Verify compute result",
            SagaAction::ComputeVerify {
                task_id: "$task_id".to_string(),
                verification_method: VerificationMethod::OutputHash,
            },
        ).with_dependencies(vec![step_idx - 1]));
        step_idx += 1;

        // Step 7: Release Payment to Provider
        steps.push(SagaStep::new(
            step_idx,
            "Release payment to provider",
            SagaAction::Transfer {
                from: requester.clone(),
                to: "$provider_did".parse().unwrap(),
                amount: "$actual_cost".parse().unwrap(),
                asset_type: budget.currency.clone(),
            },
        ).with_dependencies(vec![step_idx - 1]));
        step_idx += 1;

        // Step 8: Update Provider Trust (positive)
        steps.push(SagaStep::new(
            step_idx,
            "Update provider trust",
            SagaAction::TrustAttest {
                subject: "$provider_did".parse().unwrap(),
                claim: "compute_task_completed".to_string(),
                dimensions: TrustDelta {
                    r: 0.01,  // Reliability
                    c: 0.01,  // Competence
                    omega: 0.005,
                    ..TrustDelta::neutral()
                },
            },
        ).with_dependencies(vec![step_idx - 1]));

        Ok(steps)
    }
}
```

### 12.4 Compute Intent & Policy

```yaml
# Compute-Intent: ML-Inferenz auf verteilten Peers
intent "distributed-inference" {
  source: @identity("did:erynoa:agent:ai:claude:analyst-001")

  goal: {
    type: compute_request
    task: {
      type: ml_inference
      model: @ref("did:erynoa:model:llama-70b")
      input: {
        type: batch
        data: @ref("did:erynoa:data:customer-queries-batch-001")
        size: 1000
      }
      batch_size: 16
    }
    requirements: {
      min_cpu_cores: 8
      min_memory_mb: 32768
      gpu_required: true
      gpu_type: a100
      max_latency_ms: 60000
      geo_constraints: [
        { region: "eu", required: true }  # GDPR
      ]
    }
    max_cost: {
      amount: 100
      currency: "ERY"
    }
  }

  constraints: [
    { type: min_provider_trust, omega: 0.7 },
    { type: max_providers, count: 5 },
    { type: result_verification, method: "hash" }
  ]

  context_realm: @ref("did:erynoa:realm:acme-ml-compute")
}
```

```ecl
// Policy für Compute-Marketplace
policy ComputeMarketplace {
    // ============================================
    // Provider-Seite
    // ============================================

    if action == "offer_compute" {
        // Provider muss ausreichenden Trust haben
        require trust.omega >= 0.6,
            "Insufficient trust to offer compute";
        require trust.r >= 0.7,
            "Insufficient reliability to offer compute";

        // Kapazität muss verfügbar sein
        require has_available_capacity(sender),
            "No available compute capacity";

        // Preis muss im erlaubten Bereich liegen
        require offer.price_per_unit >= min_market_price(),
            "Price below market minimum";
        require offer.price_per_unit <= max_market_price() * 2,
            "Price too high (>2x market)";
    }

    // ============================================
    // Requester-Seite
    // ============================================

    if action == "request_compute" {
        // Budget muss ausreichend sein
        require sender.wallet_balance >= intent.max_cost,
            "Insufficient balance for compute request";

        // Task muss erlaubt sein
        require is_allowed_task_type(intent.task.type),
            "Task type not allowed in this realm";

        // Geo-Constraints müssen erfüllbar sein
        require can_satisfy_geo_constraints(intent.requirements.geo_constraints),
            "Geo constraints cannot be satisfied";
    }

    // ============================================
    // Matching
    // ============================================

    if action == "match_compute" {
        // Mindestens ein Provider muss matchen
        require count_matching_providers(requirements) >= 1,
            "No providers match requirements";

        // Provider muss Requester akzeptieren
        require provider_accepts(selected_provider, requester),
            "Provider does not accept this requester";
    }

    // ============================================
    // Execution
    // ============================================

    if action == "execute_compute" {
        // Task muss im erlaubten Zeitfenster bleiben
        require remaining_time(task) > 0,
            "Task deadline exceeded";

        // Resource-Limits einhalten
        require !resource_limit_exceeded(task),
            "Resource limits exceeded";
    }

    // ============================================
    // Verification & Payment
    // ============================================

    if action == "release_payment" {
        // Ergebnis muss verifiziert sein
        require is_result_verified(task_id),
            "Result not verified";

        // Kein Dispute offen
        require !has_open_dispute(task_id),
            "Open dispute on task";
    }

    return true;
}
```

### 12.5 Compute-Peer Konfiguration

```toml
# /etc/erynoa/compute-provider.toml
# Konfiguration für Compute-Provider

[compute_provider]
enabled = true

# Provider-Identität
provider_did = "did:erynoa:peer:compute:gpu-node-01"

# ============================================
# Kapazität
# ============================================

[compute_provider.capacity]
# CPU
cpu_cores_total = 64
cpu_cores_available = 48  # Rest für System

# Memory
memory_mb_total = 262144  # 256 GB
memory_mb_available = 245760  # 240 GB

# Storage
storage_mb_available = 2097152  # 2 TB SSD

# GPU
gpus = [
    { type = "a100", memory_gb = 80, count = 4 },
]

# ============================================
# Pricing
# ============================================

[compute_provider.pricing]
# Basis-Preis pro Compute-Unit (1 CPU-Minute)
base_price_per_unit = 0.001  # ERY

# GPU-Multiplikator
gpu_multiplier = 10.0

# Bulk-Rabatte
[[compute_provider.pricing.discounts]]
min_units = 1000
discount_percent = 5

[[compute_provider.pricing.discounts]]
min_units = 10000
discount_percent = 15

# ============================================
# Erlaubte Tasks
# ============================================

[compute_provider.allowed_tasks]
wasm_module = true
container = true
ml_inference = true
map_reduce = true
script = false  # Nur für Trusted

# ============================================
# Trust-Anforderungen
# ============================================

[compute_provider.trust_requirements]
# Minimum Trust für Requester
min_requester_trust = 0.4

# Minimum Trust für Script-Ausführung
min_script_trust = 0.9

# ============================================
# Geo-Compliance
# ============================================

[compute_provider.compliance]
# Standort
location = "eu-central-1"
jurisdiction = "DE"

# GDPR-konform
gdpr_compliant = true

# Daten-Residency
data_residency = ["eu"]
```

---

## 13. Axiom-Referenz

| Axiom       | Bereich                | Anwendung in diesem Dokument                                              |
| ----------- | ---------------------- | ------------------------------------------------------------------------- |
| **Κ8**      | Delegations-Struktur   | Shell-Capabilities werden vom Owner an Agenten delegiert mit Trust-Faktor |
| **Κ19**     | Anti-Calcification     | Zu mächtige Agenten werden erkannt und eingeschränkt                      |
| **Κ22**     | Saga-Composer          | Host-Crossing und Compute-Sagas folgen der Saga-Architektur               |
| **Κ23**     | Gateway-Policies       | ECL-Policies kontrollieren Shell/KV/Compute-Zugriff pro Peer/Realm        |
| **Κ24**     | Atomic Compensation    | Compute-Sagas haben vollständige Rollback-Pfade                           |
| **Κ20**     | Resilience             | Sandbox schützt vor State-Level-Adversaries                               |
| **RL5-RL7** | Trust-basierte Auswahl | Trust-Schwellen bestimmen erlaubte Aktionen und Realm-Beitritt            |

---

## 14. Weiterführende Dokumente

- [13-REALM-ARCHITEKTUR-ISOLATION.md](13-REALM-ARCHITEKTUR-ISOLATION.md) – Isolation zwischen Realms
- [10-IDENTITY-MULTI-DID-ARCHITEKTUR.md](10-IDENTITY-MULTI-DID-ARCHITEKTUR.md) – DID-Struktur für Agenten
- [09-TRUST-GAS-MANA-DREIEINIGKEIT.md](09-TRUST-GAS-MANA-DREIEINIGKEIT.md) – Trust-basierte Ressourcenkontrolle
- [11-SAGA-INTENT-ARCHITEKTUR.md](11-SAGA-INTENT-ARCHITEKTUR.md) – Saga-Orchestrierung und Intent-System
- [15-KV-STORE-ARCHITEKTUR.md](15-KV-STORE-ARCHITEKTUR.md) – RealmStorage und Schema-Validierung
- [documentation/concept-v2/impuls/agent-modell.md](../../concept-v2/impuls/agent-modell.md) – Agent-Lifecycle
- [documentation/system/reference/ECL-REFERENCE.md](../reference/ECL-REFERENCE.md) – ECL-Policy-Syntax

---

## 15. Zusammenfassung der erweiterten Konzepte

### Kernfunktionen

1. **Agent Shell-Zugriff**: Sandboxed Shell-Capabilities mit feingranularer Kontrolle
2. **AI-Agent Realm-Integration**: AI-Agenten mit DIDs können policy-basiert Realms beitreten
3. **Host-Crossing via Saga**: Intent → Saga erweitert von Realm↔Realm zu Realm↔Host
4. **KV-Store Zugriff**: Agenten können auf personal/shared Stores zugreifen
5. **Shared Computing Power**: Compute-Marketplace mit Trust-gewichteter Auswahl

### Sicherheitsmodell

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                           SECURITY LAYERS                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│  Layer 1: Identity (DID)                                                    │
│           └── Jede Entität hat eine kryptografische Identität              │
│                                                                             │
│  Layer 2: Trust (6D-Vector)                                                 │
│           └── Trust muss verdient werden, bestimmt Zugriff                 │
│                                                                             │
│  Layer 3: Capabilities (Κ8)                                                 │
│           └── Explizite Delegation mit Trust-Faktor-Decay                  │
│                                                                             │
│  Layer 4: Policies (ECL)                                                    │
│           └── Declarative Regeln für alle Operationen                      │
│                                                                             │
│  Layer 5: Saga (Κ22-Κ24)                                                   │
│           └── Atomic Transactions mit Compensation                          │
│                                                                             │
│  Layer 6: Sandbox (nsjail/bubblewrap)                                      │
│           └── Kernel-Level Isolation für Host-Operationen                  │
│                                                                             │
│  Layer 7: Audit Trail                                                       │
│           └── Unveränderliche Logs aller Aktionen                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Trust-Flow für AI-Agent-Operationen

```text
AI-Agent         Realm          Host           Compute-Peer
    │               │              │                 │
    │──[Join]──────►│              │                 │
    │◄──[Accept]────│              │                 │
    │               │              │                 │
    │──[KV-Read]───►│              │                 │
    │◄──[Data]──────│              │                 │
    │               │              │                 │
    │──[Shell-Intent]──────────────►│                │
    │◄──[Result]───────────────────│                 │
    │               │              │                 │
    │──[Compute-Intent]────────────────────────────►│
    │◄──[Result]────────────────────────────────────│
    │               │              │                 │
    │──[Trust-Update]──────────────────────────────►│
    │               │              │                 │
```

---

_Letzte Aktualisierung: 2025-01_
_Version: 2.0 (erweitert um AI-Agent, Host-Crossing, KV-Store, Compute)_
