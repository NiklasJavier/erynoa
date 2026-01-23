# .devcontainer/Dockerfile

# 1. Basis-Image wählen
FROM mcr.microsoft.com/devcontainers/base:ubuntu-22.04

# 2. Minimale Tools installieren (optional, aber gut für Networking/Debug)
# WICHTIG: Kein Nix, kein Entrypoint-Skript hier! Das macht alles das Feature.
RUN apt-get update && apt-get install -y --no-install-recommends \
    xz-utils \
    iproute2 \
    && rm -rf /var/lib/apt/lists/*

# DAS WAR'S. Keine weiteren Zeilen!