#!/bin/bash
# Health check script for Mirage services

# Configuration
HEALTH_CHECK_ENDPOINT="http://localhost:8081/status"
SERVICE_ENDPOINTS=(
    "http://localhost/api/v1/health"
    "http://localhost/api/v1/auth/health"
    "http://localhost/api/v1/users/health"
    "http://localhost/api/v1/modules/health"
    "http://localhost/api/v1/scans/health"
    "http://localhost/api/v1/scanner/health"
    "http://localhost/api/v1/collection/health"
    "http://localhost/api/v1/data/health"
    "http://localhost/api/v1/correlation/health"
    "http://localhost/api/v1/visualizations/health"
    "http://localhost/api/v1/reports/health"
    "http://localhost/api/v1/notifications/health"
    "http://localhost/api/v1/config/health"
)

SERVICE_NAMES=(
    "API Gateway"
    "Auth Service"
    "User Management"
    "Module Registry"
    "Scan Orchestration"
    "Scanner Coordinator"
    "Data Collection"
    "Data Storage"
    "Correlation Engine"
    "Visualization"
    "Reporting"
    "Notification"
    "Configuration"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 Mirage Platform Health Check"
echo "==============================="

# Check API Gateway status
echo -e "\n${YELLOW}Checking API Gateway Status${NC}"
gatewayStatus=$(curl -s $HEALTH_CHECK_ENDPOINT)
if [[ $gatewayStatus == *"healthy"* ]]; then
    echo -e "${GREEN}✅ API Gateway is healthy${NC}"
else
    echo -e "${RED}❌ API Gateway is not responding correctly${NC}"
    echo "$gatewayStatus"
fi

# Check individual services
echo -e "\n${YELLOW}Checking Individual Services${NC}"
echo "-------------------------------"

for i in "${!SERVICE_ENDPOINTS[@]}"; do
    service=${SERVICE_NAMES[$i]}
    endpoint=${SERVICE_ENDPOINTS[$i]}
    
    printf "%-25s" "$service:"
    response=$(curl -s -o /dev/null -w "%{http_code}" $endpoint)
    
    if [[ $response == "200" ]]; then
        echo -e "${GREEN}✅ Healthy (HTTP 200)${NC}"
    else
        echo -e "${RED}❌ Unhealthy (HTTP $response)${NC}"
    fi
done

echo -e "\n${YELLOW}Health Check Complete${NC}"
