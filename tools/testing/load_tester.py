#!/usr/bin/env python3
"""
BeeBotOS Load Tester
Performance and load testing tool for BeeBotOS APIs.
"""

import asyncio
import time
import json
import statistics
from typing import List, Dict, Callable
from dataclasses import dataclass, asdict
from concurrent.futures import ThreadPoolExecutor
import requests


@dataclass
class LoadTestResult:
    total_requests: int
    successful_requests: int
    failed_requests: int
    avg_latency_ms: float
    min_latency_ms: float
    max_latency_ms: float
    p50_latency_ms: float
    p95_latency_ms: float
    p99_latency_ms: float
    requests_per_second: float
    duration_seconds: float


class LoadTester:
    """Load testing tool for HTTP APIs."""
    
    def __init__(self, base_url: str = "http://localhost:3000"):
        self.base_url = base_url
        self.session = requests.Session()
    
    def run_test(
        self,
        endpoint: str,
        method: str = "GET",
        data: Dict = None,
        headers: Dict = None,
        concurrent_users: int = 10,
        requests_per_user: int = 100
    ) -> LoadTestResult:
        """Run a load test."""
        
        latencies = []
        success_count = 0
        fail_count = 0
        
        url = f"{self.base_url}{endpoint}"
        
        def make_request():
            try:
                start = time.time()
                
                if method == "GET":
                    response = self.session.get(url, headers=headers, timeout=30)
                elif method == "POST":
                    response = self.session.post(url, json=data, headers=headers, timeout=30)
                else:
                    raise ValueError(f"Unsupported method: {method}")
                
                latency = (time.time() - start) * 1000
                
                if response.status_code < 400:
                    return latency, True
                else:
                    return latency, False
                    
            except Exception as e:
                return 0, False
        
        def user_worker():
            user_latencies = []
            user_success = 0
            user_fail = 0
            
            for _ in range(requests_per_user):
                latency, success = make_request()
                user_latencies.append(latency)
                
                if success:
                    user_success += 1
                else:
                    user_fail += 1
            
            return user_latencies, user_success, user_fail
        
        print(f"Starting load test: {concurrent_users} users, {requests_per_user} requests each")
        print(f"Target: {url}")
        
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=concurrent_users) as executor:
            futures = [executor.submit(user_worker) for _ in range(concurrent_users)]
            
            for future in futures:
                user_latencies, user_success, user_fail = future.result()
                latencies.extend(user_latencies)
                success_count += user_success
                fail_count += user_fail
        
        duration = time.time() - start_time
        
        total_requests = len(latencies)
        
        if total_requests > 0:
            latencies.sort()
            
            result = LoadTestResult(
                total_requests=total_requests,
                successful_requests=success_count,
                failed_requests=fail_count,
                avg_latency_ms=statistics.mean(latencies),
                min_latency_ms=min(latencies),
                max_latency_ms=max(latencies),
                p50_latency_ms=latencies[int(len(latencies) * 0.5)],
                p95_latency_ms=latencies[int(len(latencies) * 0.95)],
                p99_latency_ms=latencies[int(len(latencies) * 0.99)],
                requests_per_second=total_requests / duration,
                duration_seconds=duration
            )
        else:
            result = LoadTestResult(
                total_requests=0,
                successful_requests=0,
                failed_requests=fail_count,
                avg_latency_ms=0,
                min_latency_ms=0,
                max_latency_ms=0,
                p50_latency_ms=0,
                p95_latency_ms=0,
                p99_latency_ms=0,
                requests_per_second=0,
                duration_seconds=duration
            )
        
        return result
    
    def print_report(self, result: LoadTestResult):
        """Print test report."""
        print("\n" + "=" * 60)
        print("Load Test Results")
        print("=" * 60)
        print(f"Total Requests:     {result.total_requests}")
        print(f"Successful:         {result.successful_requests}")
        print(f"Failed:             {result.failed_requests}")
        print(f"Success Rate:       {result.successful_requests / result.total_requests * 100:.1f}%")
        print("-" * 60)
        print(f"Avg Latency:        {result.avg_latency_ms:.2f} ms")
        print(f"Min Latency:        {result.min_latency_ms:.2f} ms")
        print(f"Max Latency:        {result.max_latency_ms:.2f} ms")
        print(f"P50 Latency:        {result.p50_latency_ms:.2f} ms")
        print(f"P95 Latency:        {result.p95_latency_ms:.2f} ms")
        print(f"P99 Latency:        {result.p99_latency_ms:.2f} ms")
        print("-" * 60)
        print(f"Requests/Second:    {result.requests_per_second:.2f}")
        print(f"Duration:           {result.duration_seconds:.2f} s")
        print("=" * 60)


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description="BeeBotOS Load Tester")
    parser.add_argument("--url", default="http://localhost:3000", help="Base URL")
    parser.add_argument("--endpoint", default="/health", help="API endpoint")
    parser.add_argument("--method", default="GET", choices=["GET", "POST"])
    parser.add_argument("--users", type=int, default=10, help="Number of concurrent users")
    parser.add_argument("--requests", type=int, default=100, help="Requests per user")
    parser.add_argument("--data", help="JSON data for POST requests")
    parser.add_argument("--output", help="Output results to JSON file")
    
    args = parser.parse_args()
    
    tester = LoadTester(args.url)
    
    data = json.loads(args.data) if args.data else None
    
    result = tester.run_test(
        endpoint=args.endpoint,
        method=args.method,
        data=data,
        concurrent_users=args.users,
        requests_per_user=args.requests
    )
    
    tester.print_report(result)
    
    if args.output:
        with open(args.output, "w") as f:
            json.dump(asdict(result), f, indent=2)
        print(f"\nResults saved to {args.output}")


if __name__ == "__main__":
    main()
