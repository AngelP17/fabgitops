#!/bin/bash
set -e

# FabGitOps Demo Script
# This script demonstrates the full FabGitOps workflow

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${CYAN}"
    cat << 'EOF'
    ███████╗ █████╗ ██████╗  ██████╗ ██╗████████╗ ██████╗ ██████╗ ███████╗
    ██╔════╝██╔══██╗██╔══██╗██╔════╝ ██║╚══██╔══╝██╔═══██╗██╔══██╗██╔════╝
    █████╗  ███████║██████╔╝██║  ███╗██║   ██║   ██║   ██║██████╔╝███████╗
    ██╔══╝  ██╔══██║██╔══██╗██║   ██║██║   ██║   ██║   ██║██╔═══╝ ╚════██║
    ██║     ██║  ██║██████╔╝╚██████╔╝██║   ██║   ╚██████╔╝██║     ███████║
    ╚═╝     ╚═╝  ╚═╝╚═════╝  ╚═════╝ ╚═╝   ╚═╝    ╚═════╝ ╚═╝     ╚══════╝
EOF
    echo -e "${NC}"
    echo -e "${GREEN}Industrial PLC Operator Demo${NC}"
    echo ""
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

print_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

wait_for_enter() {
    echo ""
    read -p "Press Enter to continue..."
    echo ""
}

# Check prerequisites
check_prerequisites() {
    print_step "Checking prerequisites..."
    
    command -v docker >/dev/null 2>&1 || { print_error "Docker is required but not installed."; exit 1; }
    command -v kubectl >/dev/null 2>&1 || { print_error "kubectl is required but not installed."; exit 1; }
    command -v helm >/dev/null 2>&1 || { print_error "Helm is required but not installed."; exit 1; }
    
    print_success "All prerequisites found"
}

# Build binaries
build_binaries() {
    print_step "Building FabGitOps binaries..."
    
    if [ ! -f "target/release/operator" ] || [ ! -f "target/release/fabctl" ] || [ ! -f "target/release/mock-plc" ]; then
        print_info "Building release binaries (this may take a while)..."
        cargo build --release --workspace
    else
        print_info "Binaries already built, skipping..."
    fi
    
    print_success "Binaries ready"
}

# Start observability stack
start_observability() {
    print_step "Starting Observability Stack (Prometheus + Grafana)..."
    
    docker-compose up -d
    
    print_success "Prometheus available at: http://localhost:9090"
    print_success "Grafana available at: http://localhost:3000 (admin/fabgitops)"
}

# Start mock PLC with chaos mode
start_mock_plc() {
    print_step "Starting Mock PLC with Chaos Mode..."
    
    # Kill any existing mock-plc process
    pkill -f mock-plc 2>/dev/null || true
    
    # Start mock-plc in background with chaos mode
    ./target/release/mock-plc --chaos --chaos-interval 10 --value 2500 &
    MOCK_PID=$!
    
    print_info "Mock PLC started with PID: $MOCK_PID"
    print_info "The PLC will drift randomly every 10 seconds"
    
    # Store PID for cleanup
    echo $MOCK_PID > /tmp/mock-plc.pid
    
    sleep 2
}

# Deploy operator to Kubernetes
deploy_operator() {
    print_step "Deploying FabGitOps Operator to Kubernetes..."
    
    # Apply CRD
    kubectl apply -f k8s/crd.yaml
    print_info "CRD applied"
    
    # Apply RBAC
    kubectl apply -f k8s/rbac.yaml
    print_info "RBAC applied"
    
    # Apply deployment
    kubectl apply -f k8s/deployment-local.yaml
    print_info "Operator deployment applied"
    
    # Wait for operator to be ready
    print_info "Waiting for operator to be ready..."
    kubectl rollout status deployment/fabgitops-operator --timeout=120s
    
    print_success "Operator deployed"
}

# Create sample PLCs
create_plcs() {
    print_step "Creating sample PLC resources..."
    
    kubectl apply -f k8s/sample-plc.yaml
    
    print_info "Waiting for PLC reconciliation..."
    sleep 5
    
    print_success "PLCs created"
}

# Show initial status
show_status() {
    print_step "Current PLC Status (Git vs Reality)"
    echo ""
    ./target/release/fabctl get-status
}

# Watch mode demo
watch_demo() {
    print_step "Starting Live Dashboard (Press Ctrl+C to stop watching)"
    print_info "Watch how the operator detects and corrects drift!"
    echo ""
    
    # Run watch for 30 seconds then exit
    timeout 30 ./target/release/fabctl watch --interval 2 || true
}

# Manual sync demo
manual_sync_demo() {
    print_step "Manual Sync Demonstration"
    
    print_info "Triggering manual sync for plc-line-1..."
    ./target/release/fabctl sync plc-line-1 --force
    
    sleep 2
    
    print_info "Current status after manual sync:"
    ./target/release/fabctl get-status
}

# Grafana info
show_grafana_info() {
    print_step "Grafana Dashboard"
    
    print_info "Open http://localhost:3000 in your browser"
    print_info "Login: admin / fabgitops"
    print_info "Navigate to Dashboards -> FabGitOps Dashboard"
    print_info ""
    print_info "You should see:"
    print_info "  - drift_events_total counter increasing"
    print_info "  - corrections_total counter increasing"
    print_info "  - register_value gauge changing"
    print_info "  - plc_connection_status showing connected"
}

# Cleanup
cleanup() {
    print_step "Cleaning up..."
    
    # Stop mock PLC
    if [ -f /tmp/mock-plc.pid ]; then
        kill $(cat /tmp/mock-plc.pid) 2>/dev/null || true
        rm /tmp/mock-plc.pid
    fi
    
    # Stop docker-compose
    docker-compose down
    
    # Delete K8s resources
    kubectl delete -f k8s/sample-plc.yaml 2>/dev/null || true
    kubectl delete -f k8s/deployment.yaml 2>/dev/null || true
    kubectl delete -f k8s/rbac.yaml 2>/dev/null || true
    kubectl delete -f k8s/crd.yaml 2>/dev/null || true
    
    print_success "Cleanup complete"
}

# Main demo flow
main() {
    print_banner
    
    # Parse arguments
    if [ "$1" == "cleanup" ]; then
        cleanup
        exit 0
    fi
    
    if [ "$1" == "quick" ]; then
        print_info "Running quick demo..."
        check_prerequisites
        build_binaries
        start_observability
        start_mock_plc
        deploy_operator
        create_plcs
        show_status
        echo ""
        print_success "Quick demo setup complete!"
        print_info "Run './demo.sh watch' to see the live dashboard"
        print_info "Run './demo.sh cleanup' to clean up resources"
        exit 0
    fi
    
    if [ "$1" == "watch" ]; then
        watch_demo
        exit 0
    fi
    
    # Full demo
    check_prerequisites
    wait_for_enter
    
    build_binaries
    wait_for_enter
    
    start_observability
    wait_for_enter
    
    start_mock_plc
    wait_for_enter
    
    deploy_operator
    wait_for_enter
    
    create_plcs
    wait_for_enter
    
    show_status
    wait_for_enter
    
    watch_demo
    wait_for_enter
    
    manual_sync_demo
    wait_for_enter
    
    show_grafana_info
    
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  Demo Complete!${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    print_info "To clean up resources, run: ./demo.sh cleanup"
}

# Trap Ctrl+C
trap cleanup EXIT

main "$@"
