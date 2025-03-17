-- Create scans table
CREATE TABLE scans (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(20) NOT NULL,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    priority INT NOT NULL DEFAULT 5,
    tags TEXT[] NOT NULL DEFAULT '{}',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    error_message TEXT,
    progress INT,
    estimated_completion_time TIMESTAMPTZ
);

-- Create scan targets table
CREATE TABLE scan_targets (
    id UUID PRIMARY KEY,
    scan_id UUID NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
    target_type VARCHAR(50) NOT NULL,
    value TEXT NOT NULL,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    result_count INT
);

-- Create scan modules table
CREATE TABLE scan_modules (
    id UUID PRIMARY KEY,
    scan_id UUID NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
    module_id UUID NOT NULL,
    module_name VARCHAR(255) NOT NULL,
    module_version VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL,
    parameters JSONB NOT NULL DEFAULT '{}'::jsonb,
    priority INT NOT NULL DEFAULT 1,
    depends_on UUID[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- Create indexes
CREATE INDEX idx_scans_status ON scans(status);
CREATE INDEX idx_scans_created_by ON scans(created_by);
CREATE INDEX idx_scan_targets_scan_id ON scan_targets(scan_id);
CREATE INDEX idx_scan_targets_status ON scan_targets(status);
CREATE INDEX idx_scan_modules_scan_id ON scan_modules(scan_id);
CREATE INDEX idx_scan_modules_module_id ON scan_modules(module_id);
