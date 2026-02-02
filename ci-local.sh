#!/bin/bash
#
# Local CI Script for FabGitOps
# Replicates GitHub Actions workflow locally
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Counters
PASSED=0
FAILED=0
SKIPPED=0

# Functions
print_header() {
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASSED++))
}

print_error() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAILED++))
}

print_warning() {
    echo -e "${YELLOW}[SKIP]${NC} $1"
    ((SKIPPED++))
}

print_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    print_header "CHECKING PREREQUISITES"
    
    local missing=()
    
    command -v cargo >/dev/null 2>&1 || missing+=("cargo (Rust)")
    command -v docker >/dev/null 2>&1 || missing+=("docker")
    command -v helm >/dev/null 2>&1 || missing+=("helm")
    command -v kubectl >/dev/null 2>&1 || missing+=("kubectl")
    
    if [ ${#missing[@]} -eq 0 ]; then
        print_success "All prerequisites found"
        return 0
    else
        print_error "Missing prerequisites: ${missing[*]}"
        return 1
    fi
}

# Phase 1: Lint & Test
phase_1_lint_test() {
    print_header "PHASE 1: LINT & TEST"
    
    # Check formatting
    print_step "Checking code formatting (cargo fmt)..."
    if cargo fmt -- --check 2>&1; then
        print_success "Code formatting check passed"
    else
        print_error "Code formatting check failed. Run 'cargo fmt' to fix."
        return 1
    fi
    
    # Run clippy
    print_step "Running clippy..."
    if cargo clippy --workspace -- -D warnings 2>&1; then
        print_success "Clippy check passed"
    else
        print_error "Clippy check failed"
        return 1
    fi
    
    # Run tests
    print_step "Running tests..."
    if cargo test --workspace --verbose 2>&1; then
        print_success "All tests passed"
    else
        print_error "Tests failed"
        return 1
    fi
    
    return 0
}

# Phase 2: Build Docker Images
phase_2_build_images() {
    print_header "PHASE 2: BUILD DOCKER IMAGES"
    
    # Build operator image
    print_step "Building operator Docker image..."
    if docker build -t fabgitops-operator:latest -f Dockerfile.operator . 2>&1 | tail -5; then
        print_success "Operator image built successfully"
    else
        print_error "Operator image build failed"
        return 1
    fi
    
    # Build mock-plc image
    print_step "Building mock-plc Docker image..."
    if docker build -t fabgitops-mock-plc:latest -f Dockerfile.mock-plc . 2>&1 | tail -5; then
        print_success "Mock-plc image built successfully"
    else
        print_error "Mock-plc image build failed"
        return 1
    fi
    
    return 0
}

# Phase 3: Helm Chart Validation
phase_3_helm_validation() {
    print_header "PHASE 3: HELM CHART VALIDATION"
    
    # Lint Helm chart
    print_step "Linting Helm chart..."
    if helm lint charts/fabgitops 2>&1; then
        print_success "Helm lint passed"
    else
        print_error "Helm lint failed"
        return 1
    fi
    
    # Template Helm chart
    print_step "Validating Helm templates..."
    if helm template fabgitops charts/fabgitops --debug >/dev/null 2>&1; then
        print_success "Helm template validation passed"
    else
        print_error "Helm template validation failed"
        return 1
    fi
    
    return 0
}

# Phase 4: Security Scan (optional)
phase_4_security_scan() {
    print_header "PHASE 4: SECURITY SCAN (Optional)"
    
    if ! command -v trivy >/dev/null 2>&1; then
        print_warning "Trivy not installed, skipping security scan"
        print_info "Install Trivy: https://aquasecurity.github.io/trivy/latest/getting-started/installation/"
        return 0
    fi
    
    print_step "Running Trivy security scan..."
    if trivy fs --exit-code 0 --severity HIGH,CRITICAL . 2>&1 | tail -20; then
        print_success "Security scan completed"
    else
        print_warning "Security scan found issues (review output above)"
    fi
    
    return 0
}

# Phase 5: E2E Test (optional - requires Kind)
phase_5_e2e_test() {
    print_header "PHASE 5: E2E TEST (Optional)"
    
    if ! command -v kind >/dev/null 2>&1; then
        print_warning "Kind not installed, skipping E2E test"
        print_info "Install Kind: https://kind.sigs.k8s.io/docs/user/quick-start/#installation"
        return 0
    fi
    
    print_step "Running E2E tests with Kind..."
    print_info "This will create a temporary Kind cluster"
    
    local cluster_name="fabgitops-ci-$(date +%s)"
    
    # Create Kind cluster
    if kind create cluster --name "$cluster_name" --wait 60s 2>&1; then
        print_success "Kind cluster created"
    else
        print_error "Failed to create Kind cluster"
        return 1
    fi
    
    # Load images
    print_step "Loading Docker images into Kind..."
    kind load docker-image fabgitops-operator:latest --name "$cluster_name" 2>&1
    kind load docker-image fabgitops-mock-plc:latest --name "$cluster_name" 2>&1
    
    # Deploy
    print_step "Deploying to Kind..."
    kubectl apply -f k8s/crd.yaml 2>&1
    kubectl apply -f k8s/rbac.yaml 2>&1
    kubectl apply -f k8s/deployment-local.yaml 2>&1
    kubectl apply -f k8s/mock-plc.yaml 2>&1
    
    # Wait for deployment
    print_step "Waiting for deployments..."
    if kubectl rollout status deployment/fabgitops-operator --timeout=120s 2>&1 && \
       kubectl rollout status deployment/mock-plc --timeout=120s 2>&1; then
        print_success "Deployments ready"
    else
        print_error "Deployments failed to become ready"
        kind delete cluster --name "$cluster_name" 2>/dev/null || true
        return 1
    fi
    
    # Create test PLC
    kubectl apply -f k8s/sample-plc.yaml 2>&1
    
    # Wait for reconciliation
    print_step "Waiting for PLC reconciliation..."
    sleep 10
    
    # Check status
    if kubectl get industrialplc plc-line-2 -o jsonpath='{.status.phase}' 2>/dev/null | grep -q "Connected"; then
        print_success "E2E test passed - PLC is Connected"
    else
        print_error "E2E test failed - PLC not connected"
        kubectl get industrialplc 2>&1
        kind delete cluster --name "$cluster_name" 2>/dev/null || true
        return 1
    fi
    
    # Cleanup
    print_step "Cleaning up E2E test cluster..."
    kind delete cluster --name "$cluster_name" 2>&1
    print_success "E2E test completed and cluster cleaned up"
    
    return 0
}

# Print summary
print_summary() {
    print_header "CI SUMMARY"
    
    echo -e "${GREEN}Passed:  $PASSED${NC}"
    echo -e "${RED}Failed:  $FAILED${NC}"
    echo -e "${YELLOW}Skipped: $SKIPPED${NC}"
    echo ""
    
    if [ $FAILED -eq 0 ]; then
        echo -e "${GREEN}══════════════════════════════════════════════════════════════${NC}"
        echo -e "${GREEN}  ALL CHECKS PASSED!${NC}"
        echo -e "${GREEN}══════════════════════════════════════════════════════════════${NC}"
        return 0
    else
        echo -e "${RED}══════════════════════════════════════════════════════════════${NC}"
        echo -e "${RED}  SOME CHECKS FAILED${NC}"
        echo -e "${RED}══════════════════════════════════════════════════════════════${NC}"
        return 1
    fi
}

# Main execution
main() {
    echo ""
    echo -e "${CYAN}    ███████╗ █████╗ ██████╗  ██████╗ ██╗████████╗ ██████╗ ██████╗ ███████╗${NC}"
    echo -e "${CYAN}    ██╔════╝██╔══██╗██╔══██╗██╔════╝ ██║╚══██╔══╝██╔═══██╗██╔══██╗██╔════╝${NC}"
    echo -e "${CYAN}    █████╗  ███████║██████╔╝██║  ███╗██║   ██║   ██║   ██║██████╔╝███████╗${NC}"
    echo -e "${CYAN}    ██╔══╝  ██╔══██║██╔══██╗██║   ██║██║   ██║   ██║   ██║██╔═══╝ ╚════██║${NC}"
    echo -e "${CYAN}    ██║     ██║  ██║██████╔╝╚██████╔╝██║   ██║   ╚██████╔╝██║     ███████║${NC}"
    echo -e "${CYAN}    ╚═╝     ╚═╝  ╚═╝╚═════╝  ╚═════╝ ╚═╝   ╚═╝    ╚═════╝ ╚═╝     ╚══════╝${NC}"
    echo ""
    echo -e "${CYAN}                       Local CI/CD Pipeline${NC}"
    echo ""
    
    # Parse arguments
    local run_e2e=false
    local run_security=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --e2e)
                run_e2e=true
                shift
                ;;
            --security)
                run_security=true
                shift
                ;;
            --all)
                run_e2e=true
                run_security=true
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --e2e       Run E2E tests (requires Kind)"
                echo "  --security  Run security scan (requires Trivy)"
                echo "  --all       Run all checks including E2E and security"
                echo "  --help      Show this help message"
                echo ""
                echo "Phases:"
                echo "  1. Lint & Test (cargo fmt, clippy, test)"
                echo "  2. Build Docker Images"
                echo "  3. Helm Chart Validation"
                echo "  4. Security Scan (optional)"
                echo "  5. E2E Test (optional)"
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                echo "Run '$0 --help' for usage"
                exit 1
                ;;
        esac
    done
    
    # Run phases
    check_prerequisites || exit 1
    phase_1_lint_test || exit 1
    phase_2_build_images || exit 1
    phase_3_helm_validation || exit 1
    
    if [ "$run_security" = true ]; then
        phase_4_security_scan
    fi
    
    if [ "$run_e2e" = true ]; then
        phase_5_e2e_test
    fi
    
    print_summary
    exit $FAILED
}

# Run main
main "$@"
