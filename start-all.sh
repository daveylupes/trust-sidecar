#!/bin/bash
# Start all services for Trust Sidecar testing
# Usage: ./start-all.sh [stop|status|restart]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# PID files
TRUST_SIDECAR_PID="$SCRIPT_DIR/.trust-sidecar.pid"
AD_NETWORK_PID="$SCRIPT_DIR/.ad-network.pid"
TEST_SERVER_PID="$SCRIPT_DIR/.test-server.pid"

# Ports
TRUST_SIDECAR_PORT=3000
AD_NETWORK_PORT=8081
TEST_SERVER_PORT=8080

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to check if port is in use
check_port() {
    local port=$1
    lsof -i :$port >/dev/null 2>&1
}

# Function to wait for service to be ready
wait_for_service() {
    local url=$1
    local max_attempts=30
    local attempt=0
    
    echo -n "Waiting for service at $url"
    while [ $attempt -lt $max_attempts ]; do
        if curl -s "$url" >/dev/null 2>&1; then
            echo -e " ${GREEN}ready${NC}"
            return 0
        fi
        echo -n "."
        sleep 1
        attempt=$((attempt + 1))
    done
    echo -e " ${RED}timeout${NC}"
    return 1
}

# Function to start Trust Sidecar server
start_trust_sidecar() {
    if [ -f "$TRUST_SIDECAR_PID" ]; then
        local pid=$(cat "$TRUST_SIDECAR_PID")
        if ps -p $pid >/dev/null 2>&1; then
            echo -e "${YELLOW}Trust Sidecar server already running (PID: $pid)${NC}"
            return 0
        fi
        rm -f "$TRUST_SIDECAR_PID"
    fi
    
    if check_port $TRUST_SIDECAR_PORT; then
        echo -e "${RED}Port $TRUST_SIDECAR_PORT is already in use${NC}"
        return 1
    fi
    
    echo -e "${BLUE}Starting Trust Sidecar server on port $TRUST_SIDECAR_PORT...${NC}"
    cargo run --release > "$SCRIPT_DIR/.trust-sidecar.log" 2>&1 &
    local pid=$!
    echo $pid > "$TRUST_SIDECAR_PID"
    
    wait_for_service "http://127.0.0.1:$TRUST_SIDECAR_PORT/health"
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}Trust Sidecar server started (PID: $pid)${NC}"
        echo -e "  URL: http://127.0.0.1:$TRUST_SIDECAR_PORT"
        echo -e "  Log: $SCRIPT_DIR/.trust-sidecar.log"
        return 0
    else
        echo -e "${RED}Failed to start Trust Sidecar server${NC}"
        rm -f "$TRUST_SIDECAR_PID"
        return 1
    fi
}

# Function to start Ad Network server
start_ad_network() {
    if [ -f "$AD_NETWORK_PID" ]; then
        local pid=$(cat "$AD_NETWORK_PID")
        if ps -p $pid >/dev/null 2>&1; then
            echo -e "${YELLOW}Ad Network server already running (PID: $pid)${NC}"
            return 0
        fi
        rm -f "$AD_NETWORK_PID"
    fi
    
    if check_port $AD_NETWORK_PORT; then
        echo -e "${RED}Port $AD_NETWORK_PORT is already in use${NC}"
        return 1
    fi
    
    if [ ! -d "$SCRIPT_DIR/ad-network-server" ]; then
        echo -e "${RED}Ad Network server directory not found${NC}"
        return 1
    fi
    
    echo -e "${BLUE}Starting Ad Network server on port $AD_NETWORK_PORT...${NC}"
    cd "$SCRIPT_DIR/ad-network-server"
    cargo run --release > "$SCRIPT_DIR/.ad-network.log" 2>&1 &
    local pid=$!
    cd "$SCRIPT_DIR"
    echo $pid > "$AD_NETWORK_PID"
    
    wait_for_service "http://127.0.0.1:$AD_NETWORK_PORT/health"
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}Ad Network server started (PID: $pid)${NC}"
        echo -e "  URL: http://127.0.0.1:$AD_NETWORK_PORT"
        echo -e "  Log: $SCRIPT_DIR/.ad-network.log"
        return 0
    else
        echo -e "${RED}Failed to start Ad Network server${NC}"
        rm -f "$AD_NETWORK_PID"
        return 1
    fi
}

# Function to start test server
start_test_server() {
    if [ -f "$TEST_SERVER_PID" ]; then
        local pid=$(cat "$TEST_SERVER_PID")
        if ps -p $pid >/dev/null 2>&1; then
            echo -e "${YELLOW}Test server already running (PID: $pid)${NC}"
            return 0
        fi
        rm -f "$TEST_SERVER_PID"
    fi
    
    if check_port $TEST_SERVER_PORT; then
        echo -e "${RED}Port $TEST_SERVER_PORT is already in use${NC}"
        return 1
    fi
    
    if [ ! -d "$SCRIPT_DIR/browser-extension" ]; then
        echo -e "${RED}Browser extension directory not found${NC}"
        return 1
    fi
    
    echo -e "${BLUE}Starting test server on port $TEST_SERVER_PORT...${NC}"
    cd "$SCRIPT_DIR/browser-extension"
    python3 -m http.server $TEST_SERVER_PORT > "$SCRIPT_DIR/.test-server.log" 2>&1 &
    local pid=$!
    cd "$SCRIPT_DIR"
    echo $pid > "$TEST_SERVER_PID"
    
    sleep 2
    if check_port $TEST_SERVER_PORT; then
        echo -e "${GREEN}Test server started (PID: $pid)${NC}"
        echo -e "  URL: http://localhost:$TEST_SERVER_PORT"
        echo -e "  Test page: http://localhost:$TEST_SERVER_PORT/test-page.html"
        echo -e "  Demo page: http://localhost:$TEST_SERVER_PORT/ad-network-demo.html"
        echo -e "  Log: $SCRIPT_DIR/.test-server.log"
        return 0
    else
        echo -e "${RED}Failed to start test server${NC}"
        rm -f "$TEST_SERVER_PID"
        return 1
    fi
}

# Function to stop a service
stop_service() {
    local pid_file=$1
    local service_name=$2
    
    if [ ! -f "$pid_file" ]; then
        echo -e "${YELLOW}$service_name is not running${NC}"
        return 0
    fi
    
    local pid=$(cat "$pid_file")
    if ps -p $pid >/dev/null 2>&1; then
        echo -e "${BLUE}Stopping $service_name (PID: $pid)...${NC}"
        kill $pid 2>/dev/null || true
        sleep 2
        if ps -p $pid >/dev/null 2>&1; then
            echo -e "${YELLOW}Force killing $service_name...${NC}"
            kill -9 $pid 2>/dev/null || true
        fi
        echo -e "${GREEN}$service_name stopped${NC}"
    else
        echo -e "${YELLOW}$service_name was not running${NC}"
    fi
    rm -f "$pid_file"
}

# Function to stop all services
stop_all() {
    echo -e "${BLUE}Stopping all services...${NC}"
    stop_service "$TEST_SERVER_PID" "Test server"
    stop_service "$AD_NETWORK_PID" "Ad Network server"
    stop_service "$TRUST_SIDECAR_PID" "Trust Sidecar server"
    echo -e "${GREEN}All services stopped${NC}"
}

# Function to check status
check_status() {
    echo -e "${BLUE}Service Status:${NC}"
    echo ""
    
    # Trust Sidecar
    if [ -f "$TRUST_SIDECAR_PID" ]; then
        local pid=$(cat "$TRUST_SIDECAR_PID")
        if ps -p $pid >/dev/null 2>&1; then
            if curl -s "http://127.0.0.1:$TRUST_SIDECAR_PORT/health" >/dev/null 2>&1; then
                echo -e "  Trust Sidecar: ${GREEN}Running${NC} (PID: $pid, Port: $TRUST_SIDECAR_PORT)"
            else
                echo -e "  Trust Sidecar: ${YELLOW}Running but not responding${NC} (PID: $pid)"
            fi
        else
            echo -e "  Trust Sidecar: ${RED}Not running${NC} (stale PID file)"
            rm -f "$TRUST_SIDECAR_PID"
        fi
    else
        if check_port $TRUST_SIDECAR_PORT; then
            echo -e "  Trust Sidecar: ${YELLOW}Port in use but no PID file${NC}"
        else
            echo -e "  Trust Sidecar: ${RED}Not running${NC}"
        fi
    fi
    
    # Ad Network
    if [ -f "$AD_NETWORK_PID" ]; then
        local pid=$(cat "$AD_NETWORK_PID")
        if ps -p $pid >/dev/null 2>&1; then
            if curl -s "http://127.0.0.1:$AD_NETWORK_PORT/health" >/dev/null 2>&1; then
                echo -e "  Ad Network: ${GREEN}Running${NC} (PID: $pid, Port: $AD_NETWORK_PORT)"
            else
                echo -e "  Ad Network: ${YELLOW}Running but not responding${NC} (PID: $pid)"
            fi
        else
            echo -e "  Ad Network: ${RED}Not running${NC} (stale PID file)"
            rm -f "$AD_NETWORK_PID"
        fi
    else
        if check_port $AD_NETWORK_PORT; then
            echo -e "  Ad Network: ${YELLOW}Port in use but no PID file${NC}"
        else
            echo -e "  Ad Network: ${RED}Not running${NC}"
        fi
    fi
    
    # Test Server
    if [ -f "$TEST_SERVER_PID" ]; then
        local pid=$(cat "$TEST_SERVER_PID")
        if ps -p $pid >/dev/null 2>&1; then
            if check_port $TEST_SERVER_PORT; then
                echo -e "  Test Server: ${GREEN}Running${NC} (PID: $pid, Port: $TEST_SERVER_PORT)"
            else
                echo -e "  Test Server: ${YELLOW}Running but port not accessible${NC} (PID: $pid)"
            fi
        else
            echo -e "  Test Server: ${RED}Not running${NC} (stale PID file)"
            rm -f "$TEST_SERVER_PID"
        fi
    else
        if check_port $TEST_SERVER_PORT; then
            echo -e "  Test Server: ${YELLOW}Port in use but no PID file${NC}"
        else
            echo -e "  Test Server: ${RED}Not running${NC}"
        fi
    fi
    
    echo ""
    echo -e "${BLUE}Quick Links:${NC}"
    echo -e "  Trust Sidecar API: http://127.0.0.1:$TRUST_SIDECAR_PORT"
    echo -e "  Ad Network API: http://127.0.0.1:$AD_NETWORK_PORT"
    echo -e "  Test Page: http://localhost:$TEST_SERVER_PORT/test-page.html"
    echo -e "  Demo Page: http://localhost:$TEST_SERVER_PORT/ad-network-demo.html"
}

# Main logic
case "${1:-start}" in
    start)
        echo -e "${GREEN}Starting all Trust Sidecar services...${NC}"
        echo ""
        start_trust_sidecar
        echo ""
        start_ad_network
        echo ""
        start_test_server
        echo ""
        echo -e "${GREEN}All services started!${NC}"
        echo ""
        check_status
        echo ""
        echo -e "${YELLOW}To stop all services, run: ./start-all.sh stop${NC}"
        ;;
    stop)
        stop_all
        ;;
    status)
        check_status
        ;;
    restart)
        stop_all
        sleep 2
        $0 start
        ;;
    *)
        echo "Usage: $0 [start|stop|status|restart]"
        echo ""
        echo "Commands:"
        echo "  start   - Start all services (default)"
        echo "  stop    - Stop all services"
        echo "  status  - Check status of all services"
        echo "  restart - Restart all services"
        exit 1
        ;;
esac
