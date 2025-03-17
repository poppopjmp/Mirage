-- Create integrations table
CREATE TABLE integrations (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    integration_type VARCHAR(50) NOT NULL,
    provider_id VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL,
    config JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    created_by VARCHAR(255),
    tags TEXT[] NOT NULL DEFAULT '{}',
    schedule_type VARCHAR(50) NOT NULL,
    schedule_config JSONB,
    last_execution TIMESTAMPTZ,
    next_execution TIMESTAMPTZ,
    error_message TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Create credentials table
CREATE TABLE credentials (
    id UUID PRIMARY KEY,
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    auth_type VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    encrypted_data TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ,
    last_used TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Create executions table
CREATE TABLE executions (
    id UUID PRIMARY KEY,
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    result_count INTEGER,
    error_message TEXT,
    parameters JSONB,
    target TEXT,
    execution_time_ms BIGINT
);

-- Create indexes
CREATE INDEX idx_integrations_provider_id ON integrations(provider_id);
CREATE INDEX idx_integrations_status ON integrations(status);
CREATE INDEX idx_integrations_next_execution ON integrations(next_execution);
CREATE INDEX idx_integrations_tags ON integrations USING GIN(tags);
CREATE INDEX idx_credentials_integration_id ON credentials(integration_id);
CREATE INDEX idx_executions_integration_id ON executions(integration_id);
CREATE INDEX idx_executions_started_at ON executions(started_at);

-- Create enums for better type safety
DO $$ BEGIN
    -- Create custom types for integration status
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'integration_status') THEN
        CREATE TYPE integration_status AS ENUM (
            'active', 
            'inactive', 
            'failed', 
            'pending'
        );
    END IF;

    -- Create custom types for auth type
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'auth_type') THEN
        CREATE TYPE auth_type AS ENUM (
            'none', 
            'api_key', 
            'oauth1', 
            'oauth2', 
            'basic', 
            'bearer', 
            'certificate', 
            'custom'
        );
    END IF;

    -- Create custom types for schedule type
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'schedule_type') THEN
        CREATE TYPE schedule_type AS ENUM (
            'none', 
            'once', 
            'interval', 
            'cron'
        );
    END IF;

    -- Create custom types for execution status
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'execution_status') THEN
        CREATE TYPE execution_status AS ENUM (
            'pending', 
            'running', 
            'completed', 
            'failed', 
            'cancelled'
        );
    END IF;

    -- Create custom types for integration type
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'integration_type') THEN
        CREATE TYPE integration_type AS ENUM (
            'social_media', 
            'search_engine', 
            'threat_intel', 
            'dark_web', 
            'public_database', 
            'security_tool', 
            'custom'
        );
    END IF;
END $$;
