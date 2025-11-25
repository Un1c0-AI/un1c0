# Production Vault Deployment Guide

This guide provides instructions for deploying HashiCorp Vault in production with high availability, auto-unseal, and persistent storage.

## Architecture Overview

### Production Components

1. **Vault Cluster** - 3+ nodes for HA
2. **Storage Backend** - Raft integrated storage or Consul
3. **Auto-Unseal** - Cloud KMS (AWS KMS, Azure Key Vault, GCP Cloud KMS)
4. **Load Balancer** - Distribute traffic across Vault nodes
5. **Audit Device** - File or syslog backend to external log aggregation
6. **Admin Service** - mTLS-protected admin endpoints (this PoC)
7. **nginx/TLS Terminator** - mTLS enforcement layer

## Prerequisites

- Kubernetes cluster (EKS, AKS, GKE) or VM infrastructure
- Cloud KMS service account/role
- Persistent storage (if using Raft)
- Domain name and TLS certificates
- Monitoring stack (Prometheus, Grafana)

---

## Option 1: Kubernetes Deployment with Helm

### 1. Add Vault Helm Repository

```bash
helm repo add hashicorp https://helm.releases.hashicorp.com
helm repo update
```

### 2. Create values.yaml

```yaml
# vault-values.yaml
global:
  enabled: true
  tlsDisable: false

server:
  image:
    repository: "hashicorp/vault"
    tag: "1.15.3"
  
  # High Availability with Raft storage
  ha:
    enabled: true
    replicas: 3
    raft:
      enabled: true
      setNodeId: true
      config: |
        ui = true
        
        listener "tcp" {
          tls_disable = 0
          address = "[::]:8200"
          cluster_address = "[::]:8201"
          tls_cert_file = "/vault/userconfig/vault-server-tls/tls.crt"
          tls_key_file = "/vault/userconfig/vault-server-tls/tls.key"
        }
        
        storage "raft" {
          path = "/vault/data"
        }
        
        # Auto-unseal using AWS KMS
        seal "awskms" {
          region     = "us-east-1"
          kms_key_id = "your-kms-key-id"
        }
        
        # Enable Prometheus metrics
        telemetry {
          prometheus_retention_time = "30s"
          disable_hostname = true
        }
        
        service_registration "kubernetes" {}
  
  # Persistent storage for Raft
  dataStorage:
    enabled: true
    size: 10Gi
    storageClass: gp3
    accessMode: ReadWriteOnce
  
  # Audit logging
  auditStorage:
    enabled: true
    size: 10Gi
    storageClass: gp3
    accessMode: ReadWriteOnce
  
  # Service account for cloud KMS access
  serviceAccount:
    create: true
    name: "vault"
    annotations:
      eks.amazonaws.com/role-arn: "arn:aws:iam::ACCOUNT:role/vault-kms-role"
  
  # Resources
  resources:
    requests:
      memory: 256Mi
      cpu: 250m
    limits:
      memory: 512Mi
      cpu: 500m

# Enable UI
ui:
  enabled: true
  serviceType: "ClusterIP"

# Injector for secrets injection
injector:
  enabled: true
  replicas: 2
```

### 3. Deploy Vault

```bash
# Create namespace
kubectl create namespace vault

# Create TLS secret for Vault
kubectl create secret tls vault-server-tls \
  --cert=vault-server.crt \
  --key=vault-server.key \
  -n vault

# Install Vault
helm install vault hashicorp/vault \
  --namespace vault \
  --values vault-values.yaml

# Wait for pods to be ready
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=vault -n vault --timeout=300s
```

### 4. Initialize Vault Cluster

```bash
# Initialize the first Vault pod
kubectl exec -n vault vault-0 -- vault operator init \
  -key-shares=5 \
  -key-threshold=3 \
  -format=json > vault-init-keys.json

# IMPORTANT: Store vault-init-keys.json securely (encrypted, offline)
# It contains recovery keys even when using auto-unseal

# Join other nodes to the Raft cluster
kubectl exec -n vault vault-1 -- vault operator raft join http://vault-0.vault-internal:8200
kubectl exec -n vault vault-2 -- vault operator raft join http://vault-0.vault-internal:8200

# Verify cluster status
kubectl exec -n vault vault-0 -- vault operator raft list-peers
```

### 5. Enable Audit Logging

```bash
# Login with root token
VAULT_TOKEN=$(jq -r '.root_token' vault-init-keys.json)

kubectl exec -n vault vault-0 -- vault login $VAULT_TOKEN

# Enable file audit device
kubectl exec -n vault vault-0 -- vault audit enable file \
  file_path=/vault/audit/audit.log
```

---

## Option 2: VM/Docker Deployment

### 1. Create Vault Configuration

```hcl
# /etc/vault.d/vault.hcl
ui = true

listener "tcp" {
  address       = "0.0.0.0:8200"
  tls_cert_file = "/opt/vault/tls/vault-server.crt"
  tls_key_file  = "/opt/vault/tls/vault-server.key"
}

storage "raft" {
  path    = "/opt/vault/data"
  node_id = "node1"  # Unique per node
  
  retry_join {
    leader_api_addr = "https://vault-1.example.com:8200"
  }
  retry_join {
    leader_api_addr = "https://vault-2.example.com:8200"
  }
}

# Auto-unseal with AWS KMS
seal "awskms" {
  region     = "us-east-1"
  kms_key_id = "your-kms-key-id"
}

# Telemetry
telemetry {
  prometheus_retention_time = "30s"
  disable_hostname          = true
}

api_addr = "https://vault-0.example.com:8200"
cluster_addr = "https://vault-0.example.com:8201"
```

### 2. Start Vault Service

```bash
# Create systemd service
cat <<EOF > /etc/systemd/system/vault.service
[Unit]
Description=HashiCorp Vault
Documentation=https://www.vaultproject.io/docs/
Requires=network-online.target
After=network-online.target

[Service]
User=vault
Group=vault
ExecStart=/usr/local/bin/vault server -config=/etc/vault.d/vault.hcl
ExecReload=/bin/kill -HUP \$MAINPID
KillMode=process
Restart=on-failure
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

# Start and enable
systemctl daemon-reload
systemctl enable vault
systemctl start vault
```

### 3. Initialize and Join Cluster

```bash
# On node1 (first node)
vault operator init -format=json > vault-init-keys.json

# On node2 and node3
vault operator raft join https://vault-1.example.com:8200
```

---

## Deploying Admin Service with mTLS

### 1. Update docker-compose.yml for Production

```yaml
version: '3.8'

services:
  admin-service:
    build: ./admin_service
    environment:
      VAULT_ADDR: "https://vault.example.com:8200"
      VAULT_TOKEN: "${ADMIN_VAULT_TOKEN}"  # From secret manager
      APPROLE_NAME: "master-key-approle"
      ADMIN_LOG_DIR: "/logs"
    volumes:
      - admin-logs:/logs
      - /var/run/docker.sock:/var/run/docker.sock:ro
    networks:
      - vault-net
    restart: unless-stopped

  nginx:
    image: nginx:stable
    ports:
      - "8443:8443"
    volumes:
      - ./nginx/mutual_tls.conf:/etc/nginx/conf.d/default.conf:ro
      - ./certs/production:/etc/nginx/certs:ro
    networks:
      - vault-net
    restart: unless-stopped
    depends_on:
      - admin-service

volumes:
  admin-logs:

networks:
  vault-net:
    driver: bridge
```

### 2. Generate Production Certificates

```bash
# Use your organization's PKI or Let's Encrypt for server certs
# For client certs, use your internal CA

# Example with internal CA
openssl genrsa -out production-ca.key 4096
openssl req -x509 -new -nodes -key production-ca.key \
  -sha256 -days 3650 -out production-ca.crt \
  -subj "/CN=Production Vault CA/O=YourOrg"

# Generate client cert for GitHub Actions runners
openssl genrsa -out github-runner-client.key 2048
openssl req -new -key github-runner-client.key \
  -out github-runner-client.csr \
  -subj "/CN=github-runner/O=YourOrg"

openssl x509 -req -in github-runner-client.csr \
  -CA production-ca.crt -CAkey production-ca.key \
  -CAcreateserial -out github-runner-client.crt \
  -days 365 -sha256
```

---

## Monitoring and Alerting

### 1. Prometheus Scrape Config

```yaml
scrape_configs:
  - job_name: 'vault'
    metrics_path: '/v1/sys/metrics'
    params:
      format: ['prometheus']
    scheme: https
    tls_config:
      ca_file: /etc/prometheus/vault-ca.crt
    bearer_token: 'your-vault-token'
    static_configs:
      - targets:
        - vault-0.example.com:8200
        - vault-1.example.com:8200
        - vault-2.example.com:8200
```

### 2. Key Metrics to Monitor

- `vault_core_unsealed` - Unseal status
- `vault_runtime_alloc_bytes` - Memory usage
- `vault_runtime_num_goroutines` - Goroutine count
- `vault_audit_log_request` - Audit log throughput
- `vault_token_count` - Active token count
- `vault_expire_num_leases` - Lease count

### 3. Grafana Dashboard

Import the official Vault dashboard:
- Dashboard ID: 12904
- Or build custom dashboards using the metrics above

---

## Security Hardening Checklist

### Vault Configuration

- [ ] TLS enabled on all listeners
- [ ] Auto-unseal configured with cloud KMS
- [ ] Audit logging enabled to external system
- [ ] Recovery keys stored offline, encrypted
- [ ] Resource quotas configured
- [ ] Telemetry exported to monitoring system
- [ ] UI disabled or behind authentication
- [ ] Vault agent injector for workload identity

### Admin Service

- [ ] mTLS enforcement for all endpoints
- [ ] Rate limiting configured (nginx)
- [ ] Application logs sent to SIEM
- [ ] Break-glass token stored offline
- [ ] Certificate rotation automated
- [ ] AppRole accessor cleanup scheduled
- [ ] Master key rotation scheduled quarterly
- [ ] Metrics endpoint exposed for monitoring

### Network

- [ ] Vault cluster in private subnet
- [ ] Admin service not publicly exposed
- [ ] Load balancer health checks configured
- [ ] DDoS protection enabled
- [ ] Network policies restricting access
- [ ] VPN or private link for admin access

### Operations

- [ ] Disaster recovery plan documented
- [ ] Backup and restore tested
- [ ] Break-glass procedures documented
- [ ] On-call rotation established
- [ ] Runbooks created for common issues
- [ ] Change management process in place
- [ ] Regular security audits scheduled

---

## Backup and Disaster Recovery

### Raft Storage Snapshots

```bash
# Create snapshot
vault operator raft snapshot save backup-$(date +%Y%m%d).snap

# Restore from snapshot
vault operator raft snapshot restore backup-20250101.snap
```

### Automated Backup Script

```bash
#!/bin/bash
# /usr/local/bin/vault-backup.sh

VAULT_ADDR="https://vault.example.com:8200"
VAULT_TOKEN="your-token"
BACKUP_DIR="/backups/vault"
RETENTION_DAYS=30

# Create snapshot
vault operator raft snapshot save \
  "$BACKUP_DIR/vault-$(date +%Y%m%d-%H%M%S).snap"

# Upload to S3
aws s3 cp "$BACKUP_DIR/vault-$(date +%Y%m%d-%H%M%S).snap" \
  s3://your-backup-bucket/vault/

# Cleanup old backups
find "$BACKUP_DIR" -name "vault-*.snap" -mtime +$RETENTION_DAYS -delete
```

### Cron Schedule

```bash
# Daily backup at 2 AM
0 2 * * * /usr/local/bin/vault-backup.sh
```

---

## Migration from PoC to Production

### 1. Export PoC Data

```bash
# Export master key
vault kv get -format=json kv/master_key > master_key_backup.json

# Export policies
vault policy list | while read policy; do
  vault policy read "$policy" > "policy_${policy}.hcl"
done

# Export AppRole configuration
vault read -format=json auth/approle/role/master-key-approle > approle_config.json
```

### 2. Import to Production

```bash
# Create KV v2 mount
vault secrets enable -path=kv kv-v2

# Import master key
vault kv put kv/master_key @master_key_backup.json

# Import policies
for policy in policy_*.hcl; do
  name=$(basename "$policy" .hcl | sed 's/policy_//')
  vault policy write "$name" "$policy"
done

# Create AppRole
vault auth enable approle
vault write auth/approle/role/master-key-approle @approle_config.json
```

### 3. Update GitHub Secrets

```bash
# Get new Vault address and role_id
cd vault
./configure_secrets.sh

# Update workflows to use production Vault address
# Set VAULT_ADDR environment variable in workflows
```

---

## Troubleshooting

### Vault Unsealed but Unavailable

```bash
# Check cluster status
vault status

# Check raft peers
vault operator raft list-peers

# Check for split brain
vault operator raft autopilot state
```

### Performance Issues

```bash
# Check request metrics
vault read sys/metrics

# Enable debug logging temporarily
vault operator raft autopilot set-config \
  -cleanup-dead-servers=true \
  -dead-server-last-contact-threshold=10m
```

### Admin Service Not Responding

```bash
# Check nginx logs
docker logs vault-nginx-1

# Check admin service logs
tail -f vault/admin_service/logs/admin_service.log

# Verify mTLS handshake
openssl s_client -connect localhost:8443 \
  -cert certs/client.crt -key certs/client.key
```

---

## Additional Resources

- [Vault Production Hardening](https://developer.hashicorp.com/vault/tutorials/operations/production-hardening)
- [Vault Reference Architecture](https://developer.hashicorp.com/vault/tutorials/operations/reference-architecture)
- [Auto-Unseal Configuration](https://developer.hashicorp.com/vault/docs/concepts/seal)
- [Raft Storage Backend](https://developer.hashicorp.com/vault/docs/configuration/storage/raft)
- [Vault Helm Chart](https://github.com/hashicorp/vault-helm)
