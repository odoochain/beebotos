# DevOps Engineer

## Description

Autonomous DevOps automation agent for CI/CD, infrastructure management, and deployment.

## Capabilities

```yaml
functions:
  - name: deploy_service
    description: Deploy service to Kubernetes
  - name: manage_pipeline
    description: Manage CI/CD pipelines
  - name: monitor_infrastructure
    description: Monitor infrastructure health
  - name: auto_scale
    description: Auto-scale based on metrics
```

## Configuration

```yaml
config:
  default_provider: kubernetes
  auto_rollback: true
  health_check_interval: 30
```
