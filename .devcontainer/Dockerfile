# Base DevContainer with Nix pre-installed
# Build once: docker build -t godstack-devcontainer .devcontainer/
FROM mcr.microsoft.com/devcontainers/base:ubuntu

# Install apt packages in one layer (cached)
RUN apt-get update && apt-get install -y --no-install-recommends \
    xz-utils \
    ca-certificates \
    curl \
    git \
    direnv \
    && rm -rf /var/lib/apt/lists/*

# Install Nix (multi-user)
RUN curl -L https://nixos.org/nix/install | sh -s -- --daemon --yes \
    && echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

# Add direnv hooks
RUN echo 'eval "$(direnv hook bash)"' >> /etc/bash.bashrc \
    && echo 'eval "$(direnv hook zsh)"' >> /etc/zsh/zshrc

# Nix needs this for multi-user mode
COPY --chmod=755 <<EOF /usr/local/bin/start-nix-daemon.sh
#!/bin/bash
if [ -e '/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh' ]; then
  . '/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh'
fi
exec "\$@"
EOF

ENTRYPOINT ["/usr/local/bin/start-nix-daemon.sh"]
CMD ["sleep", "infinity"]
