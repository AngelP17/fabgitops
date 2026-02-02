# ADR 001: Use Rust for Kubernetes Operator

## Status

**Accepted**

## Context

FabGitOps requires a Kubernetes operator to manage Industrial PLC resources. The operator needs to:
- Reconcile PLC state continuously with minimal latency
- Handle Modbus TCP communication with industrial hardware
- Run reliably for extended periods without restarts
- Provide real-time metrics for observability

The two primary language options for Kubernetes operators are:
1. **Go** - The "official" language with kubebuilder/controller-runtime
2. **Rust** - Using kube-rs library

## Decision

We will use **Rust** (with kube-rs) to build the FabGitOps operator.

## Consequences

### Positive

1. **No Garbage Collection Pauses**: Rust's ownership model eliminates GC stop-the-world pauses, ensuring consistent reconciliation loop timing critical for industrial control systems.

2. **Memory Safety**: Compile-time guarantees prevent memory leaks and use-after-free bugs that could cause operator crashes in long-running deployments.

3. **Performance**: Native performance with zero-cost abstractions means lower CPU and memory usage compared to Go's runtime overhead.

4. **Type Safety**: Rust's type system catches errors at compile time, reducing runtime failures in production.

5. **Modbus Ecosystem**: The `tokio-modbus` crate provides robust async Modbus TCP support.

### Negative

1. **Learning Curve**: Team members may need to learn Rust's ownership model and async patterns.

2. **Ecosystem Maturity**: While kube-rs is production-ready, it has fewer examples than controller-runtime.

3. **Build Times**: Rust compile times are longer than Go, potentially impacting CI/CD speed.

## Alternatives Considered

### Go (controller-runtime)

**Pros:**
- Official Kubernetes SDK
- Extensive documentation and community
- Fast compile times

**Cons:**
- GC pauses can disrupt timing-sensitive reconciliation
- Runtime panics possible due to nil pointer dereferences
- Higher memory usage

**Verdict:** Rejected due to GC unpredictability in industrial control scenarios.

### Python (kopf)

**Pros:**
- Rapid development
- Easy to read

**Cons:**
- GIL limits concurrency
- High memory usage
- Slow execution

**Verdict:** Rejected due to performance and reliability concerns.

## References

- [kube-rs documentation](https://docs.rs/kube/)
- [Rust Kubernetes Operators](https://rust-on-nails.com/docs/kubernetes-operators/)
- [Comparing Go and Rust for Kubernetes Operators](https://www.youtube.com/watch?v=IaK4P0dE3zQ)
