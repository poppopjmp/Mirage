#!/bin/bash
# Deploy API Gateway for Mirage

# Check if running with sudo
if [ "$EUID" -ne 0 ]; then
  echo "Please run as root or with sudo"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
NGINX_CONFIG_DIR="$PROJECT_ROOT/config/nginx"
NGINX_CACHE_DIR="/var/cache/nginx/mirage"

echo "🚀 Deploying Mirage API Gateway"
echo "============================="

# Step 1: Check if Nginx is installed
echo "Checking Nginx installation..."
if ! command -v nginx &> /dev/null; then
    echo "Nginx not found. Installing..."
    apt-get update
    apt-get install -y nginx
    if [ $? -ne 0 ]; then
        echo "❌ Failed to install Nginx. Aborting."
        exit 1
    fi
    echo "✅ Nginx installed successfully."
else
    echo "✅ Nginx already installed."
fi

# Step 2: Create cache directories if they don't exist
echo "Setting up Nginx cache directories..."
mkdir -p "$NGINX_CACHE_DIR"
mkdir -p "$NGINX_CACHE_DIR/temp"
chown -R nginx:nginx "$NGINX_CACHE_DIR"
chmod -R 755 "$NGINX_CACHE_DIR"
echo "✅ Cache directories created."

# Step 3: Copy configuration files
echo "Copying Nginx configuration files..."
cp "$NGINX_CONFIG_DIR/nginx.conf" /etc/nginx/nginx.conf
cp "$NGINX_CONFIG_DIR/default.conf" /etc/nginx/conf.d/default.conf
echo "✅ Configuration files copied."

# Step 4: Test configuration
echo "Testing Nginx configuration..."
nginx -t
if [ $? -ne 0 ]; then
    echo "❌ Nginx configuration test failed. Aborting."
    exit 1
else
    echo "✅ Nginx configuration is valid."
fi

# Step 5: Restart Nginx
echo "Restarting Nginx..."
systemctl restart nginx
if [ $? -ne 0 ]; then
    echo "❌ Failed to restart Nginx. Aborting."
    exit 1
else
    echo "✅ Nginx restarted successfully."
fi

# Step 6: Enable Nginx to start on boot
echo "Enabling Nginx service to start on boot..."
systemctl enable nginx
echo "✅ Nginx enabled."

echo "🎉 API Gateway deployment complete!"
echo "You can access the Mirage platform at http://localhost"
