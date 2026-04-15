# Skill: IoT Device Manager

## Description

Autonomous IoT device management and monitoring agent.

## Capabilities

```yaml
functions:
  - name: monitor_devices
    description: Monitor device status
  - name: control_device
    description: Send commands to devices
  - name: collect_data
    description: Collect sensor data
  - name: detect_anomalies
    description: Detect anomalous behavior
```

## Configuration

```yaml
config:
  protocols: ["mqtt", "coap", "http"]
  auto_discovery: true
  alert_threshold: 0.9
```
