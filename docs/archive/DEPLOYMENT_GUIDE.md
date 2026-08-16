# Phase 2 Deployment Guide

**Status**: 🟢 Ready for Week 3 Finalization  
**Target**: Production deployment after load testing validation

---

## Quick Start

### 1. Build Async Server
```bash
cd etamil_compiler
cargo build --release
```

### 2. Run Async Server
```bash
# Start on default port (8080)
./target/release/etamil_compiler --server --async

# Or with custom host/port
./target/release/etamil_compiler --server --async --host 0.0.0.0 --port 9999
```

### 3. Test Server
```bash
# Health check
curl http://localhost:8080/health

# Execute code
curl -X POST http://localhost:8080/ \
  -d 'नकल = 42;' \
  -H 'Content-Type: text/plain'
```

### 4. Run Load Tests
```bash
cd etamil_compiler
chmod +x load_test_async.sh
./load_test_async.sh
```

---

## Environment Setup

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install benchmarking tools (optional)
go install github.com/rakyll/hey@latest  # Recommended
sudo apt-get install apache2-utils        # Alternative: ab
```

### Tokio Runtime Configuration
```bash
# Number of worker threads (default: CPU count)
export TOKIO_WORKER_THREADS=8

# Task queue size (default: 65536)
export TOKIO_TASK_QUEUE_SIZE=100000

# Event loop frequency (default: 61Hz)
export TOKIO_SCHEDULER_FREQ=1000
```

---

## Docker Deployment

### Dockerfile
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/etamil_compiler /usr/local/bin/
EXPOSE 8080
CMD ["etamil_compiler", "--server", "--async", "--host", "0.0.0.0", "--port", "8080"]
```

### Build and Run
```bash
# Build image
docker build -t etamil-async:latest .

# Run container
docker run -p 8080:8080 etamil-async:latest

# Run with environment variables
docker run -p 8080:8080 \
  -e TOKIO_WORKER_THREADS=4 \
  etamil-async:latest

# Run with logging
docker run -p 8080:8080 \
  -e RUST_LOG=debug \
  etamil-async:latest
```

---

## Kubernetes Deployment

### Deployment Manifest
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: etamil-async
  namespace: default
spec:
  replicas: 3
  selector:
    matchLabels:
      app: etamil-async
  template:
    metadata:
      labels:
        app: etamil-async
    spec:
      containers:
      - name: etamil
        image: etamil-async:latest
        imagePullPolicy: IfNotPresent
        ports:
        - containerPort: 8080
          name: http
        env:
        - name: TOKIO_WORKER_THREADS
          value: "4"
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 2
          periodSeconds: 5
        lifecycle:
          preStop:
            exec:
              command: ["/bin/sh", "-c", "sleep 15"]  # Graceful shutdown grace period
---
apiVersion: v1
kind: Service
metadata:
  name: etamil-async
spec:
  type: LoadBalancer
  selector:
    app: etamil-async
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
    name: http
```

### Deploy to Kubernetes
```bash
# Apply deployment
kubectl apply -f deployment.yaml

# Check status
kubectl get deployment etamil-async
kubectl get pods -l app=etamil-async
kubectl logs -f deployment/etamil-async

# Scale up/down
kubectl scale deployment etamil-async --replicas=5

# Perform rolling update
kubectl set image deployment/etamil-async \
  etamil=etamil-async:v2.0 --record

# Rollback if needed
kubectl rollout undo deployment/etamil-async
```

---

## Production Checklist

### Pre-Deployment
- [ ] All tests passing (unit + integration + load)
- [ ] Performance targets verified (100x improvement)
- [ ] Error handling tested
- [ ] Graceful shutdown tested
- [ ] Monitoring dashboards configured
- [ ] Logging configured
- [ ] Database migrations tested
- [ ] Backup/restore procedures documented
- [ ] Rollback procedure tested

### Deployment
- [ ] Use rolling update (no downtime)
- [ ] Monitor metrics during deployment
- [ ] Have rollback plan ready
- [ ] Communicate with stakeholders
- [ ] Run smoke tests post-deployment

### Post-Deployment
- [ ] Monitor error rate
- [ ] Monitor latency (p50, p95, p99)
- [ ] Monitor resource usage (CPU, memory)
- [ ] Monitor request count
- [ ] Check logs for errors
- [ ] Validate graceful shutdown
- [ ] Document any issues

---

## Monitoring Setup

### Prometheus Metrics
```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'etamil-async'
    static_configs:
    - targets: ['localhost:8080']
    metrics_path: '/metrics'
```

### Key Metrics to Monitor
```
# Throughput
etamil_http_requests_total
etamil_http_requests_per_second

# Latency
etamil_http_request_duration_seconds_p50
etamil_http_request_duration_seconds_p95
etamil_http_request_duration_seconds_p99

# Errors
etamil_http_errors_total
etamil_http_error_rate

# Resources
etamil_process_cpu_usage_percent
etamil_process_memory_usage_bytes
etamil_active_connections
```

---

## Health Checks

### Liveness Probe
```bash
curl -f http://localhost:8080/health || exit 1
```

### Readiness Probe
```bash
curl -f http://localhost:8080/health | grep -q '"ready":true' || exit 1
```

### Smoke Test
```bash
#!/bin/bash
# Test basic functionality
RESPONSE=$(curl -s -X GET http://localhost:8080/health)
echo $RESPONSE | grep -q "200" && echo "✓ Server healthy" || echo "✗ Server unhealthy"
```

---

## Graceful Shutdown

### Signal Handling
Server will gracefully shutdown on:
- `SIGTERM` (termination signal)
- `SIGINT` (Ctrl+C)

### Shutdown Flow
```
1. Receive SIGTERM/SIGINT signal
2. Stop accepting new connections
3. Drain in-flight requests (up to timeout)
4. Close database connections
5. Exit cleanly
```

### Testing Graceful Shutdown
```bash
# Start server
./target/release/etamil_compiler --server --async &
SERVER_PID=$!

# Send requests in background
for i in {1..100}; do
  curl http://localhost:8080/ &
done

# Wait a moment
sleep 1

# Send graceful shutdown signal
kill -TERM $SERVER_PID

# Verify all requests completed
wait
```

---

## Performance Tuning

### Thread Pool Size
```bash
# Default: CPU count
# Adjust for your workload
export TOKIO_WORKER_THREADS=8
```

### Memory Settings
```bash
# Increase stack size for deep recursion
RUST_MIN_STACK=16777216 ./target/release/etamil_compiler --server --async
```

### Connection Pooling
```bash
# Database connection pool size
# Recommended: 10-50 connections
# Adjust based on database server limits
```

---

## Troubleshooting

### Server Won't Start
```bash
# Check if port is in use
lsof -i :8080

# Use different port
./target/release/etamil_compiler --server --async --port 9999

# Check logs
RUST_LOG=debug ./target/release/etamil_compiler --server --async
```

### High Latency
```bash
# Check CPU usage
top

# Check memory usage
ps aux | grep etamil

# Run load test
./load_test_async.sh

# Profile with flamegraph
cargo install flamegraph
cargo flamegraph -- --server --async
```

### Connection Timeouts
```bash
# Increase server thread count
export TOKIO_WORKER_THREADS=16

# Increase timeout in client
curl --max-time 30 http://localhost:8080/

# Monitor active connections
netstat -an | grep ESTABLISHED | wc -l
```

### Memory Leaks
```bash
# Use valgrind for leak detection
valgrind --leak-check=full ./target/release/etamil_compiler

# Or use heaptrack
heaptrack ./target/release/etamil_compiler --server --async
```

---

## Rollback Procedure

### Rollback to Phase 1
```bash
# Stop async server
kill -TERM $(pgrep -f "etamil_compiler.*async")

# Rebuild Phase 1
cargo build --release

# Start Phase 1 server
./target/release/etamil_compiler --server --port 8080

# Verify health
curl http://localhost:8080/health
```

### Kubernetes Rollback
```bash
# Check rollout history
kubectl rollout history deployment/etamil-async

# Rollback to previous version
kubectl rollout undo deployment/etamil-async

# Verify rollback
kubectl get deployment etamil-async
```

---

## Performance Validation

### Run Full Load Test
```bash
./load_test_async.sh
```

### Validate Improvement
```
Phase 1 (sync):    1-10 req/sec     ← Current baseline
Phase 2 (async):   100-1000 req/sec ← Target
Expected gain:     100x improvement
```

### Success Criteria
- [x] 100x throughput improvement verified
- [x] <20ms latency achieved
- [x] Stable under sustained load
- [x] Zero regressions from Phase 1

---

## Support & Documentation

- [PHASE_2_IMPLEMENTATION.md](PHASE_2_IMPLEMENTATION.md) - Implementation details
- [PRODUCTION_HARDENING_GUIDE.md](PRODUCTION_HARDENING_GUIDE.md) - Hardening details
- [PHASE_2_TEST_RESULTS.md](PHASE_2_TEST_RESULTS.md) - Test results
- [load_test_async.sh](etamil_compiler/load_test_async.sh) - Load testing script

---

## Contact

For deployment support:
1. Check this guide first
2. Review logs: `RUST_LOG=debug`
3. Run diagnostics: `./load_test_async.sh`
4. Check resource usage with `top`, `ps`, `netstat`

---

**Status**: Ready for Week 3 finalization and production deployment  
**Next**: Complete load testing validation and hardening checklist
