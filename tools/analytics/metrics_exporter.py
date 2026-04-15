#!/usr/bin/env python3
"""
BeeBotOS Metrics Exporter
Exports system metrics to Prometheus and other monitoring systems.
"""

import json
import time
import psutil
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional
from dataclasses import dataclass, asdict
from collections import defaultdict
import threading


@dataclass
class MetricPoint:
    """Single metric data point."""
    name: str
    value: float
    timestamp: float
    labels: Dict[str, str]
    
    def to_prometheus(self) -> str:
        """Convert to Prometheus format."""
        labels_str = ",".join([f'{k}="{v}"' for k, v in self.labels.items()])
        if labels_str:
            return f"{name}{{{labels_str}}} {value}"
        return f"{name} {value}"


class MetricsCollector:
    """Collects system metrics."""
    
    def __init__(self, collection_interval: int = 15):
        self.interval = collection_interval
        self.metrics: List[MetricPoint] = []
        self.running = False
        self._lock = threading.Lock()
        self._thread: Optional[threading.Thread] = None
    
    def start(self):
        """Start metrics collection."""
        if self.running:
            return
        
        self.running = True
        self._thread = threading.Thread(target=self._collect_loop)
        self._thread.daemon = True
        self._thread.start()
    
    def stop(self):
        """Stop metrics collection."""
        self.running = False
        if self._thread:
            self._thread.join(timeout=5)
    
    def _collect_loop(self):
        """Main collection loop."""
        while self.running:
            try:
                self._collect_system_metrics()
                self._collect_process_metrics()
                time.sleep(self.interval)
            except Exception as e:
                print(f"Metrics collection error: {e}")
    
    def _collect_system_metrics(self):
        """Collect system-level metrics."""
        timestamp = time.time()
        
        # CPU metrics
        cpu_percent = psutil.cpu_percent(interval=1)
        self._add_metric("system_cpu_usage_percent", cpu_percent, {}, timestamp)
        
        # Memory metrics
        memory = psutil.virtual_memory()
        self._add_metric("system_memory_used_bytes", memory.used, {}, timestamp)
        self._add_metric("system_memory_available_bytes", memory.available, {}, timestamp)
        self._add_metric("system_memory_usage_percent", memory.percent, {}, timestamp)
        
        # Disk metrics
        disk = psutil.disk_usage('/')
        self._add_metric("system_disk_used_bytes", disk.used, {}, timestamp)
        self._add_metric("system_disk_free_bytes", disk.free, {}, timestamp)
        self._add_metric("system_disk_usage_percent", disk.percent, {}, timestamp)
        
        # Network metrics
        net_io = psutil.net_io_counters()
        self._add_metric("system_network_bytes_sent", net_io.bytes_sent, {}, timestamp)
        self._add_metric("system_network_bytes_recv", net_io.bytes_recv, {}, timestamp)
        
        # Load average
        load_avg = psutil.getloadavg()
        self._add_metric("system_load_average_1m", load_avg[0], {}, timestamp)
    
    def _collect_process_metrics(self):
        """Collect process-level metrics."""
        timestamp = time.time()
        
        for proc in psutil.process_iter(['pid', 'name', 'cpu_percent', 'memory_info']):
            try:
                labels = {"process": proc.info['name'], "pid": str(proc.info['pid'])}
                self._add_metric(
                    "process_cpu_percent",
                    proc.info['cpu_percent'],
                    labels,
                    timestamp
                )
                self._add_metric(
                    "process_memory_bytes",
                    proc.info['memory_info'].rss,
                    labels,
                    timestamp
                )
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue
    
    def _add_metric(self, name: str, value: float, labels: Dict[str, str], timestamp: float):
        """Add a metric point."""
        with self._lock:
            self.metrics.append(MetricPoint(name, value, timestamp, labels))
            
            # Keep only last hour of metrics
            cutoff = timestamp - 3600
            self.metrics = [m for m in self.metrics if m.timestamp > cutoff]
    
    def get_metrics(self, name_filter: Optional[str] = None) -> List[MetricPoint]:
        """Get collected metrics."""
        with self._lock:
            metrics = self.metrics.copy()
        
        if name_filter:
            metrics = [m for m in metrics if name_filter in m.name]
        
        return metrics
    
    def export_prometheus(self) -> str:
        """Export metrics in Prometheus format."""
        lines = []
        
        # Group by metric name
        grouped = defaultdict(list)
        for metric in self.metrics:
            grouped[metric.name].append(metric)
        
        for name, metrics in grouped.items():
            lines.append(f"# TYPE {name} gauge")
            for metric in metrics:
                lines.append(metric.to_prometheus())
            lines.append("")
        
        return "\n".join(lines)
    
    def export_json(self) -> str:
        """Export metrics as JSON."""
        return json.dumps([asdict(m) for m in self.metrics], indent=2)


class PrometheusExporter:
    """Prometheus metrics HTTP exporter."""
    
    def __init__(self, collector: MetricsCollector, port: int = 9090):
        self.collector = collector
        self.port = port
    
    def start(self):
        """Start HTTP server."""
        try:
            from http.server import HTTPServer, BaseHTTPRequestHandler
            
            collector = self.collector
            
            class Handler(BaseHTTPRequestHandler):
                def do_GET(self):
                    if self.path == '/metrics':
                        self.send_response(200)
                        self.send_header('Content-Type', 'text/plain')
                        self.end_headers()
                        self.wfile.write(collector.export_prometheus().encode())
                    elif self.path == '/':
                        self.send_response(200)
                        self.send_header('Content-Type', 'text/html')
                        self.end_headers()
                        self.wfile.write(b'<h1>BeeBotOS Metrics</h1><a href="/metrics">Metrics</a>')
                    else:
                        self.send_response(404)
                        self.end_headers()
                
                def log_message(self, format, *args):
                    pass  # Suppress logs
            
            server = HTTPServer(('0.0.0.0', self.port), Handler)
            print(f"Prometheus exporter listening on port {self.port}")
            server.serve_forever()
            
        except ImportError:
            print("http.server not available")


class MetricsReporter:
    """Reports metrics to external systems."""
    
    def __init__(self, collector: MetricsCollector):
        self.collector = collector
        self.reporters: List[callable] = []
    
    def add_reporter(self, reporter: callable):
        """Add a reporter function."""
        self.reporters.append(reporter)
    
    def report(self):
        """Report current metrics."""
        metrics = self.collector.get_metrics()
        for reporter in self.reporters:
            try:
                reporter(metrics)
            except Exception as e:
                print(f"Reporter error: {e}")


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description="BeeBotOS Metrics Exporter")
    parser.add_argument("--port", type=int, default=9090, help="Prometheus port")
    parser.add_argument("--interval", type=int, default=15, help="Collection interval")
    parser.add_argument("--output", type=str, help="Output file for JSON export")
    
    args = parser.parse_args()
    
    # Start collector
    collector = MetricsCollector(args.interval)
    collector.start()
    
    print(f"Metrics collector started (interval={args.interval}s)")
    
    # Start Prometheus exporter
    exporter = PrometheusExporter(collector, args.port)
    
    try:
        exporter.start()
    except KeyboardInterrupt:
        print("\nShutting down...")
        collector.stop()
        
        if args.output:
            Path(args.output).write_text(collector.export_json())
            print(f"Metrics exported to {args.output}")


if __name__ == "__main__":
    main()
