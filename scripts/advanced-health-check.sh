#!/bin/bash
# Advanced health check script for Mirage services with detailed diagnostics

# Configuration
API_GATEWAY="http://localhost"
HEALTH_CHECK_ENDPOINT="http://localhost:8081/status"
LOG_DIR="/var/log/nginx"
TIMEOUT=5 # seconds for curl requests

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Service endpoints and names
declare -A SERVICE_INFO=(
    ["auth"]="Auth Service|http://localhost/api/v1/auth/health"
    ["users"]="User Management|http://localhost/api/v1/users/health"
    ["modules"]="Module Registry|http://localhost/api/v1/modules/health"
    ["scans"]="Scan Orchestration|http://localhost/api/v1/scans/health"
    ["scanner"]="Scanner Coordinator|http://localhost/api/v1/scanner/health"
    ["collection"]="Data Collection|http://localhost/api/v1/collection/health"
    ["data"]="Data Storage|http://localhost/api/v1/data/health"
    ["correlation"]="Correlation Engine|http://localhost/api/v1/correlation/health"
    ["visualizations"]="Visualization|http://localhost/api/v1/visualizations/health"
    ["reports"]="Reporting|http://localhost/api/v1/reports/health"
    ["notifications"]="Notification|http://localhost/api/v1/notifications/health"
    ["config"]="Configuration Service|http://localhost/api/v1/config/health"
)

# Header function
print_header() {
    echo
    echo -e "${BLUE}=====================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}=====================================${NC}"
}

# Check if a command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# Check if we're running with sudo/root for log access
check_permissions() {
    if [ "$EUID" -ne 0 ] && [ ! -r "$LOG_DIR/error.log" ]; then
        echo -e "${YELLOW}⚠️  Warning: Not running with sufficient permissions to read logs.${NC}"
        echo -e "${YELLOW}   Some diagnostics may not be available.${NC}"
        echo -e "${YELLOW}   Consider running with 'sudo' for full diagnostics.${NC}"
        echo
    fi
}

# Check API Gateway status
check_gateway() {
    print_header "API Gateway Status Check"
    
    echo -e "${YELLOW}Testing API Gateway connectivity...${NC}"
    gateway_response=$(curl -s -o /dev/null -w "%{http_code}" $API_GATEWAY -m $TIMEOUT)
    
    if [[ "$gateway_response" =~ ^[23] ]]; then
        echo -e "${GREEN}✅ API Gateway is responding (HTTP $gateway_response)${NC}"
        
        # Check the detailed health status
        echo -e "\n${YELLOW}Checking detailed gateway status...${NC}"
        status_response=$(curl -s $HEALTH_CHECK_ENDPOINT -m $TIMEOUT)
        
        if [[ $status_response == *"healthy"* ]]; then
            echo -e "${GREEN}✅ Gateway health status: healthy${NC}"
            echo -e "${YELLOW}Reported services status:${NC}"
            
            # Extract and display service status
            for service in "${!SERVICE_INFO[@]}"; do
                service_status=$(echo $status_response | grep -o "\"$service\": *\"[^\"]*\"" | cut -d'"' -f4)
                if [[ "$service_status" == "up" ]]; then
                    echo -e "  ${GREEN}✓ $service${NC}"
                else
                    echo -e "  ${RED}✗ $service ($service_status)${NC}"
                fi
            done
        else
            echo -e "${RED}❌ Gateway health check failed${NC}"
            echo "$status_response"
        fi
    else
        echo -e "${RED}❌ API Gateway is not responding correctly (HTTP $gateway_response)${NC}"
    fi
}

# Check individual services
check_services() {
    print_header "Individual Service Health Checks"
    
    # Table header
    printf "%-30s %-15s %-15s %-15s\n" "Service" "Status" "Response Time" "HTTP Code"
    printf "%-30s %-15s %-15s %-15s\n" "-------" "------" "-------------" "---------"
    
    for service in "${!SERVICE_INFO[@]}"; do
        IFS='|' read -r name endpoint <<< "${SERVICE_INFO[$service]}"
        
        printf "%-30s " "$name"
        
        # Get response with timing info
        timing=$(curl -s -o /dev/null -w "%{http_code};%{time_total}" $endpoint -m $TIMEOUT)
        IFS=';' read -r status time <<< "$timing"
        
        # Format time to ms
        time_ms=$(echo "$time * 1000" | bc | awk '{printf "%.2f ms", $0}')
        
        if [[ "$status" == "200" ]]; then
            printf "${GREEN}%-15s${NC} " "Healthy"
            printf "%-15s " "$time_ms"
            printf "%-15s\n" "$status"
        else
            printf "${RED}%-15s${NC} " "Unhealthy"
            printf "%-15s " "$time_ms"
            printf "%-15s\n" "$status"
        fi
    done
}

# Check Nginx logs for errors
check_logs() {
    print_header "Recent Error Logs"
    
    if [ -r "$LOG_DIR/error.log" ]; then
        echo -e "${YELLOW}Last 5 errors from Nginx error log:${NC}"
        grep -i "error" "$LOG_DIR/error.log" | tail -n 5
        
        echo -e "\n${YELLOW}5 most recent requests with status code ≥ 400:${NC}"
        grep -E 'HTTP/(1\.1|2\.0)" [4-5][0-9]{2}' "$LOG_DIR/access.log" | tail -n 5
    else
        echo -e "${RED}Cannot read Nginx logs. Try running with sudo.${NC}"
    fi
}

# Main execution
main() {
    echo "🔍 Mirage Platform Advanced Health Check"
    echo "========================================"
    echo "Executed at: $(date)"
    echo "System: $(uname -a)"
    
    check_permissions
    check_gateway
    check_services
    check_logs
    
    print_header "Health Check Summary"
    echo -e "${YELLOW}API Gateway:${NC} $(curl -s -o /dev/null -w "%{http_code}" $API_GATEWAY -m 2)"
    
    # Count healthy vs unhealthy services
    healthy=0
    unhealthy=0
    
    for service in "${!SERVICE_INFO[@]}"; do
        IFS='|' read -r name endpoint <<< "${SERVICE_INFO[$service]}"
        status=$(curl -s -o /dev/null -w "%{http_code}" $endpoint -m 2)
        
        if [[ "$status" == "200" ]]; then
            ((healthy++))
        else
            ((unhealthy++))
        fi
    done
    
    echo -e "${GREEN}✅ Healthy services: $healthy${NC}"
    echo -e "${RED}❌ Unhealthy services: $unhealthy${NC}"
    
    if [[ $unhealthy -gt 0 ]]; then
        echo -e "\n${YELLOW}Recommendation:${NC} Check the logs of unhealthy services or restart them."
    fi
    
    echo -e "\nHealth check completed at $(date)"
}

# Execute main function
main
