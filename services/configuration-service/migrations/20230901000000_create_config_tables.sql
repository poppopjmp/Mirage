-- Create config namespaces table
CREATE TABLE config_namespaces (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- Create default namespaces
INSERT INTO config_namespaces (id, name, description, created_at, updated_at)
VALUES 
    ('11111111-1111-1111-1111-111111111111', 'system', 'System configuration settings', NOW(), NOW()),
    ('22222222-2222-2222-2222-222222222222', 'modules', 'Module-specific configurations', NOW(), NOW()),
    ('33333333-3333-3333-3333-333333333333', 'user', 'User preferences and settings', NOW(), NOW()),
    ('44444444-4444-4444-4444-444444444444', 'scan', 'Scan configuration settings', NOW(), NOW()),
    ('55555555-5555-5555-5555-555555555555', 'security', 'Security-related settings', NOW(), NOW());

-- Create config items table
CREATE TABLE config_items (
    id UUID PRIMARY KEY,
    key VARCHAR(255) NOT NULL,
    namespace VARCHAR(255) NOT NULL,
    value JSONB NOT NULL,
    value_type VARCHAR(50) NOT NULL,
    description TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    is_secret BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    created_by VARCHAR(255),
    updated_by VARCHAR(255),
    schema JSONB,
    tags TEXT[] NOT NULL DEFAULT '{}',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    UNIQUE(key, namespace)
);

-- Create config versions table (history tracking)
CREATE TABLE config_versions (
    id UUID PRIMARY KEY,
    config_id UUID NOT NULL REFERENCES config_items(id) ON DELETE CASCADE,
    value JSONB NOT NULL,
    version INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    created_by VARCHAR(255),
    comment TEXT,
    UNIQUE(config_id, version)
);

-- Create audit logs table
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    action VARCHAR(50) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    user_id VARCHAR(255),
    timestamp TIMESTAMPTZ NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::jsonb,
    change_summary TEXT,
    service VARCHAR(100)
);

-- Create indexes
CREATE INDEX idx_config_items_namespace ON config_items(namespace);
CREATE INDEX idx_config_items_key ON config_items(key);
CREATE INDEX idx_config_items_tags ON config_items USING GIN(tags);
CREATE INDEX idx_config_versions_config_id ON config_versions(config_id);
CREATE INDEX idx_audit_logs_entity_id ON audit_logs(entity_id);
CREATE INDEX idx_audit_logs_entity_type ON audit_logs(entity_type);
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);

-- Insert some initial default configuration
INSERT INTO config_items (
    id, key, namespace, value, value_type, 
    description, version, is_secret, created_at, updated_at,
    created_by, tags
)
VALUES (
    '66666666-6666-6666-6666-666666666666',
    'default_scan_timeout',
    'system',
    '300'::jsonb,
    'integer',
    'Default timeout for scan operations in seconds',
    1,
    FALSE,
    NOW(),
    NOW(),
    'system',
    ARRAY['scan', 'timeout', 'default']
);

-- Add initial version record
INSERT INTO config_versions (
    id, config_id, value, version, created_at, created_by, comment
)
VALUES (
    '77777777-7777-7777-7777-777777777777',
    '66666666-6666-6666-6666-666666666666',
    '300'::jsonb,
    1,
    NOW(),
    'system',
    'Initial configuration'
);
