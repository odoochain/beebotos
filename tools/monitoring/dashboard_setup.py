#!/usr/bin/env python3
"""
BeeBotOS Monitoring Dashboard Setup
Sets up Grafana dashboards and Prometheus rules
"""

import json
import yaml
import argparse
from pathlib import Path

DASHBOARD_TEMPLATE = {
    "dashboard": {
        "title": "BeeBotOS Overview",
        "tags": ["beebotos", "monitoring"],
        "timezone": "UTC",
        "panels": [
            {
                "id": 1,
                "title": "Request Rate",
                "type": "stat",
                "targets": [{
                    "expr": "rate(gateway_requests_total[5m])",
                    "legendFormat": "{{status}}"
                }],
                "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0}
            },
            {
                "id": 2,
                "title": "Response Time",
                "type": "graph",
                "targets": [{
                    "expr": "histogram_quantile(0.99, rate(gateway_request_duration_seconds_bucket[5m]))",
                    "legendFormat": "p99"
                }],
                "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0}
            },
            {
                "id": 3,
                "title": "Active Agents",
                "type": "stat",
                "targets": [{
                    "expr": "agent_active_total",
                    "legendFormat": "Active"
                }],
                "gridPos": {"h": 8, "w": 8, "x": 0, "y": 8}
            },
            {
                "id": 4,
                "title": "Memory Usage",
                "type": "graph",
                "targets": [{
                    "expr": "process_resident_memory_bytes",
                    "legendFormat": "{{job}}"
                }],
                "gridPos": {"h": 8, "w": 16, "x": 8, "y": 8}
            },
            {
                "id": 5,
                "title": "A2A Messages",
                "type": "graph",
                "targets": [{
                    "expr": "rate(a2a_messages_total[5m])",
                    "legendFormat": "{{type}}"
                }],
                "gridPos": {"h": 8, "w": 12, "x": 0, "y": 16}
            },
            {
                "id": 6,
                "title": "DAO Proposals",
                "type": "table",
                "targets": [{
                    "expr": "dao_proposals",
                    "format": "table"
                }],
                "gridPos": {"h": 8, "w": 12, "x": 12, "y": 16}
            }
        ]
    }
}

ALERT_RULES = {
    "groups": [{
        "name": "beebotos",
        "rules": [
            {
                "alert": "HighErrorRate",
                "expr": "rate(gateway_requests_total{status=~\"5..\"}[5m]) > 0.1",
                "for": "5m",
                "labels": {"severity": "critical"},
                "annotations": {
                    "summary": "High error rate detected",
                    "description": "Error rate is above 10%"
                }
            },
            {
                "alert": "HighLatency",
                "expr": "histogram_quantile(0.99, rate(gateway_request_duration_seconds_bucket[5m])) > 1",
                "for": "5m",
                "labels": {"severity": "warning"},
                "annotations": {
                    "summary": "High latency detected",
                    "description": "P99 latency is above 1 second"
                }
            },
            {
                "alert": "AgentDown",
                "expr": "up{job=\"agent-runtime\"} == 0",
                "for": "1m",
                "labels": {"severity": "critical"},
                "annotations": {
                    "summary": "Agent runtime is down",
                    "description": "Agent runtime has been down for more than 1 minute"
                }
            },
            {
                "alert": "LowMemory",
                "expr": "process_resident_memory_bytes / process_virtual_memory_max_bytes > 0.9",
                "for": "5m",
                "labels": {"severity": "warning"},
                "annotations": {
                    "summary": "Low memory available",
                    "description": "Memory usage is above 90%"
                }
            }
        ]
    }]
}

def create_dashboard(output_dir: str):
    """Create Grafana dashboard JSON"""
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)
    
    dashboard_file = output_path / "beebotos-dashboard.json"
    dashboard_file.write_text(json.dumps(DASHBOARD_TEMPLATE, indent=2))
    print(f"Created: {dashboard_file}")

def create_alert_rules(output_dir: str):
    """Create Prometheus alert rules"""
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)
    
    alerts_file = output_path / "beebotos-alerts.yaml"
    yaml.dump(ALERT_RULES, alerts_file.open('w'), default_flow_style=False)
    print(f"Created: {alerts_file}")

def create_datasource_config(output_dir: str):
    """Create Grafana datasource configuration"""
    config = {
        "apiVersion": 1,
        "datasources": [
            {
                "name": "Prometheus",
                "type": "prometheus",
                "url": "http://prometheus:9090",
                "access": "proxy",
                "isDefault": True
            },
            {
                "name": "Loki",
                "type": "loki",
                "url": "http://loki:3100",
                "access": "proxy"
            }
        ]
    }
    
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)
    
    ds_file = output_path / "datasources.yaml"
    yaml.dump(config, ds_file.open('w'), default_flow_style=False)
    print(f"Created: {ds_file}")

def main():
    parser = argparse.ArgumentParser(description="BeeBotOS Monitoring Setup")
    parser.add_argument("-o", "--output", default="./monitoring", 
                       help="Output directory")
    parser.add_argument("--dashboard", action="store_true", 
                       help="Create dashboard only")
    parser.add_argument("--alerts", action="store_true", 
                       help="Create alerts only")
    
    args = parser.parse_args()
    
    create_all = not (args.dashboard or args.alerts)
    
    if create_all or args.dashboard:
        create_dashboard(args.output)
        create_datasource_config(args.output)
    
    if create_all or args.alerts:
        create_alert_rules(args.output)
    
    print(f"\n✅ Monitoring setup complete!")
    print(f"📁 Output: {Path(args.output).absolute()}")

if __name__ == "__main__":
    main()
