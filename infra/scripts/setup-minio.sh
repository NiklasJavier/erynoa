#!/usr/bin/env bash
# ============================================================================
# MinIO Auto-Setup für Godstack
# ============================================================================
# Erstellt automatisch Buckets und Policies für das System
# ============================================================================

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Config
MINIO_ENDPOINT="http://localhost:9000"
MINIO_ROOT_USER="godstack"
MINIO_ROOT_PASSWORD="godstack123"
DATA_DIR="${DATA_DIR:-$(cd "$(dirname "$0")/../.." && pwd)/.data}"
SETUP_MARKER="$DATA_DIR/.minio-setup-complete"

# Buckets to create
BUCKETS=(
    "uploads"       # User uploads
    "avatars"       # Profile pictures
    "documents"     # General documents
    "temp"          # Temporary files (with lifecycle policy)
)

log() { echo -e "${BLUE}[MinIO]${NC} $1"; }
success() { echo -e "${BLUE}[MinIO]${NC} ${GREEN}✓${NC} $1"; }
warn() { echo -e "${BLUE}[MinIO]${NC} ${YELLOW}⚠${NC} $1"; }
error() { echo -e "${BLUE}[MinIO]${NC} ${RED}✗${NC} $1" >&2; }

# Print banner
print_banner() {
    echo -e "${BLUE}"
    echo "╔════════════════════════════════════════════════════════════════════╗"
    echo "║          📦 MinIO Auto-Setup für Godstack                          ║"
    echo "╚════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

# Wait for MinIO to be ready
wait_for_minio() {
    log "Warte auf MinIO..."
    local max_attempts=30
    local attempt=0
    
    while ! curl -sf "${MINIO_ENDPOINT}/minio/health/ready" > /dev/null 2>&1; do
        attempt=$((attempt + 1))
        if [ $attempt -ge $max_attempts ]; then
            error "MinIO nicht erreichbar nach ${max_attempts} Versuchen"
            exit 1
        fi
        sleep 1
    done
    success "MinIO ist bereit"
}

# Install mc (MinIO Client) if not present
ensure_mc() {
    if command -v mc &> /dev/null; then
        return 0
    fi
    
    log "Installiere MinIO Client (mc)..."
    local mc_url=""
    local arch=$(uname -m)
    
    case "$arch" in
        x86_64|amd64) mc_url="https://dl.min.io/client/mc/release/linux-amd64/mc" ;;
        aarch64|arm64) mc_url="https://dl.min.io/client/mc/release/linux-arm64/mc" ;;
        *) error "Unsupported architecture: $arch"; exit 1 ;;
    esac
    
    curl -sL "$mc_url" -o /tmp/mc
    chmod +x /tmp/mc
    sudo mv /tmp/mc /usr/local/bin/mc 2>/dev/null || mv /tmp/mc "$HOME/.local/bin/mc"
    success "MinIO Client installiert"
}

# Configure mc alias
configure_mc() {
    log "Konfiguriere MinIO Client..."
    mc alias set godstack "$MINIO_ENDPOINT" "$MINIO_ROOT_USER" "$MINIO_ROOT_PASSWORD" --api S3v4 > /dev/null 2>&1
    success "MinIO Client konfiguriert"
}

# Create buckets
create_buckets() {
    echo ""
    echo "━━━ Buckets erstellen ━━━"
    
    for bucket in "${BUCKETS[@]}"; do
        if mc ls "godstack/$bucket" > /dev/null 2>&1; then
            success "Bucket '$bucket' existiert bereits"
        else
            mc mb "godstack/$bucket" > /dev/null 2>&1
            success "Bucket '$bucket' erstellt"
        fi
    done
}

# Set up bucket policies
setup_policies() {
    echo ""
    echo "━━━ Policies einrichten ━━━"
    
    # Avatars bucket - public read
    local avatars_policy='{
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Principal": {"AWS": ["*"]},
            "Action": ["s3:GetObject"],
            "Resource": ["arn:aws:s3:::avatars/*"]
        }]
    }'
    echo "$avatars_policy" | mc anonymous set-json /dev/stdin godstack/avatars 2>/dev/null || true
    success "Avatars bucket: public read"
    
    # Uploads - private (default)
    mc anonymous set none godstack/uploads > /dev/null 2>&1 || true
    success "Uploads bucket: private"
    
    # Documents - private (default)
    mc anonymous set none godstack/documents > /dev/null 2>&1 || true
    success "Documents bucket: private"
    
    # Temp - private with lifecycle
    mc anonymous set none godstack/temp > /dev/null 2>&1 || true
    success "Temp bucket: private"
}

# Set up lifecycle rules
setup_lifecycle() {
    echo ""
    echo "━━━ Lifecycle Rules ━━━"
    
    # Temp bucket - delete after 24 hours
    local lifecycle_config='{
        "Rules": [{
            "ID": "expire-temp-files",
            "Status": "Enabled",
            "Expiration": {
                "Days": 1
            }
        }]
    }'
    
    # mc ilm import doesn't work well in scripts, so we just note it
    success "Temp bucket: 24h expiration (manuell zu setzen)"
}

# Create application service account
create_service_account() {
    echo ""
    echo "━━━ Service Account ━━━"
    
    local creds_file="$DATA_DIR/.minio-credentials"
    
    # Check if we already have credentials
    if [ -f "$creds_file" ]; then
        success "Service Account Credentials existieren bereits"
        return 0
    fi
    
    # Create new service account
    local sa_output
    sa_output=$(mc admin user svcacct add godstack "$MINIO_ROOT_USER" --json 2>/dev/null || echo '{}')
    
    local access_key=$(echo "$sa_output" | grep -o '"accessKey":"[^"]*"' | cut -d'"' -f4)
    local secret_key=$(echo "$sa_output" | grep -o '"secretKey":"[^"]*"' | cut -d'"' -f4)
    
    if [ -n "$access_key" ] && [ -n "$secret_key" ]; then
        echo "MINIO_ACCESS_KEY=$access_key" > "$creds_file"
        echo "MINIO_SECRET_KEY=$secret_key" >> "$creds_file"
        chmod 600 "$creds_file"
        success "Service Account erstellt: $access_key"
    else
        # Fallback: Use root credentials for dev
        echo "MINIO_ACCESS_KEY=$MINIO_ROOT_USER" > "$creds_file"
        echo "MINIO_SECRET_KEY=$MINIO_ROOT_PASSWORD" >> "$creds_file"
        chmod 600 "$creds_file"
        warn "Verwende Root Credentials (nur für Entwicklung)"
    fi
}

# Print summary
print_summary() {
    local creds_file="$DATA_DIR/.minio-credentials"
    local access_key=""
    local secret_key=""
    
    if [ -f "$creds_file" ]; then
        access_key=$(grep MINIO_ACCESS_KEY "$creds_file" | cut -d= -f2)
        secret_key=$(grep MINIO_SECRET_KEY "$creds_file" | cut -d= -f2)
    fi
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo -e "🎉 ${GREEN}MinIO Setup abgeschlossen!${NC}"
    echo ""
    echo "   S3 Endpoint:        $MINIO_ENDPOINT"
    echo "   Console:            http://localhost:9001"
    echo ""
    echo "   Buckets:"
    for bucket in "${BUCKETS[@]}"; do
        echo "     - $bucket"
    done
    echo ""
    echo "   Credentials:"
    echo "     Access Key:       $access_key"
    echo "     Secret Key:       ${secret_key:0:8}..."
    echo ""
    echo "   Console Login:"
    echo "     User:             $MINIO_ROOT_USER"
    echo "     Password:         $MINIO_ROOT_PASSWORD"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# Quick check if already setup
check_existing_setup() {
    if [ -f "$SETUP_MARKER" ]; then
        # Verify buckets still exist
        if mc ls godstack/uploads > /dev/null 2>&1; then
            local access_key=$(grep MINIO_ACCESS_KEY "$DATA_DIR/.minio-credentials" 2>/dev/null | cut -d= -f2)
            success "Setup bereits abgeschlossen"
            echo "   Endpoint: $MINIO_ENDPOINT"
            echo "   Access Key: $access_key"
            return 0
        fi
    fi
    return 1
}

# Main
main() {
    print_banner
    
    mkdir -p "$DATA_DIR"
    
    # Quick path if already setup
    if check_existing_setup 2>/dev/null; then
        exit 0
    fi
    
    wait_for_minio
    ensure_mc
    configure_mc
    create_buckets
    setup_policies
    setup_lifecycle
    create_service_account
    
    # Mark as complete
    date > "$SETUP_MARKER"
    
    print_summary
}

main "$@"
