-- ============================================================================
-- Migration: 20240101000001_create_users_table.sql
-- Erstellt die Users Tabelle mit ZITADEL Mapping
-- ============================================================================

-- Extensions (falls nicht bereits vorhanden)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Users Tabelle
CREATE TABLE IF NOT EXISTS users (
    -- Interne ID (für Foreign Keys)
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- ZITADEL User ID (externer Identity Provider)
    zitadel_id VARCHAR(255) UNIQUE NOT NULL,
    
    -- Basis-Informationen (von ZITADEL synchronisiert)
    email VARCHAR(255),
    
    -- Interne Business-Rolle (unabhängig von ZITADEL Rollen)
    internal_role VARCHAR(50) NOT NULL DEFAULT 'user',
    
    -- Flexible Metadaten für Business-Logik
    metadata JSONB NOT NULL DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indizes
CREATE INDEX IF NOT EXISTS idx_users_zitadel_id ON users(zitadel_id);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email) WHERE email IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_users_internal_role ON users(internal_role);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);

-- GIN Index für JSONB Metadaten (für schnelle Abfragen)
CREATE INDEX IF NOT EXISTS idx_users_metadata ON users USING GIN (metadata);

-- Trigger für automatisches updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE 'plpgsql';

DROP TRIGGER IF EXISTS update_users_updated_at ON users;
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Kommentar zur Tabelle
COMMENT ON TABLE users IS 'User accounts linked to ZITADEL identity provider';
COMMENT ON COLUMN users.zitadel_id IS 'External user ID from ZITADEL';
COMMENT ON COLUMN users.internal_role IS 'Application-specific role (user, admin, manager)';
COMMENT ON COLUMN users.metadata IS 'Flexible JSONB field for business-specific data';
