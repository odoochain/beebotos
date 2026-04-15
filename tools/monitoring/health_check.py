#!/usr/bin/env python3
"""
BeeBotOS Health Check Tool
Monitors system health and performance metrics.
"""

import sys
import time
import json
import requests
from typing import Dict, List, Optional
from dataclasses import dataclass
from datetime import datetime


@dataclass
class HealthStatus:
    component: str
    status: str  # healthy, degraded, down
    latency_ms: float
    last_check: str
    details: Dict


class HealthChecker:
    """System health checker for BeeBotOS."""
    
    def __init__(self, base_url: str = "http://localhost:3000"):
        self.base_url = base_url
        self.components = [
            "api",
            "agent-runtime",
            "memory-store",
            "blockchain-connector",
        ]
        
    def check_api(self) -> HealthStatus:
        """Check API gateway health."""
        start = time.time()
        try:
            response = requests.get(f"{self.base_url}/health", timeout=5)
            latency = (time.time() - start) * 1000
            
            if response.status_code == 200:
                return HealthStatus(
                    component="api",
                    status="healthy",
                    latency_ms=latency,
                    last_check=datetime.now().isoformat(),
                    details=response.json()
                )
            else:
                return HealthStatus(
                    component="api",
                    status="degraded",
                    latency_ms=latency,
                    last_check=datetime.now().isoformat(),
                    details={"status_code": response.status_code}
                )
        except Exception as e:
            return HealthStatus(
                component="api",
                status="down",
                latency_ms=(time.time() - start) * 1000,
                last_check=datetime.now().isoformat(),
                details={"error": str(e)}
            )
    
    def check_all(self) -> List[HealthStatus]:
        """Run all health checks."""
        results = []
        results.append(self.check_api())
        return results
    
    def print_report(self, results: List[HealthStatus]):
        """Print health check report."""
        print("\n" + "=" * 60)
        print("BeeBotOS Health Check Report")
        print("=" * 60)
        print(f"Time: {datetime.now().isoformat()}")
        print("-" * 60)
        
        for result in results:
            status_icon = "✅" if result.status == "healthy" else "⚠️" if result.status == "degraded" else "❌"
            print(f"{status_icon} {result.component:20} | {result.status:10} | {result.latency_ms:.1f}ms")
        
        print("-" * 60)
        healthy = sum(1 for r in results if r.status == "healthy")
        print(f"Summary: {healthy}/{len(results)} components healthy")
        print("=" * 60)


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description="BeeBotOS Health Checker")
    parser.add_argument("--url", default="http://localhost:3000", help="API base URL")
    parser.add_argument("--watch", action="store_true", help="Continuous monitoring")
    parser.add_argument("--interval", type=int, default=30, help="Check interval in seconds")
    parser.add_argument("--json", action="store_true", help="Output as JSON")
    
    args = parser.parse_args()
    
    checker = HealthChecker(args.url)
    
    if args.watch:
        print(f"Starting continuous monitoring (interval: {args.interval}s)...")
        try:
            while True:
                results = checker.check_all()
                checker.print_report(results)
                time.sleep(args.interval)
        except KeyboardInterrupt:
            print("\nMonitoring stopped.")
    else:
        results = checker.check_all()
        
        if args.json:
            print(json.dumps([{
                "component": r.component,
                "status": r.status,
                "latency_ms": r.latency_ms,
                "last_check": r.last_check,
                "details": r.details
            } for r in results], indent=2))
        else:
            checker.print_report(results)
        
        # Exit with error code if any component is down
        if any(r.status == "down" for r in results):
            sys.exit(1)


if __name__ == "__main__":
    main()
