-- Initialize all service databases
CREATE DATABASE IF NOT EXISTS mirage_auth;
CREATE DATABASE IF NOT EXISTS mirage_users;
CREATE DATABASE IF NOT EXISTS mirage_scans;
CREATE DATABASE IF NOT EXISTS mirage_notifications;
CREATE DATABASE IF NOT EXISTS mirage_integration;
CREATE DATABASE IF NOT EXISTS mirage_config;
CREATE DATABASE IF NOT EXISTS mirage_discovery;

-- Grant permissions
GRANT ALL PRIVILEGES ON DATABASE mirage_auth TO mirage;
GRANT ALL PRIVILEGES ON DATABASE mirage_users TO mirage;
GRANT ALL PRIVILEGES ON DATABASE mirage_scans TO mirage;
GRANT ALL PRIVILEGES ON DATABASE mirage_notifications TO mirage;
GRANT ALL PRIVILEGES ON DATABASE mirage_integration TO mirage;
GRANT ALL PRIVILEGES ON DATABASE mirage_config TO mirage;
GRANT ALL PRIVILEGES ON DATABASE mirage_discovery TO mirage;