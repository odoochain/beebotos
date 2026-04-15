#!/usr/bin/env python3
"""BeeBotOS Code Analysis Tool"""

import os
import json
from pathlib import Path
from collections import defaultdict

def analyze_project():
    """Analyze project structure and generate report"""
    stats = {
        'total_files': 0,
        'total_lines': 0,
        'by_extension': defaultdict(int),
        'by_directory': defaultdict(int),
        'largest_files': []
    }
    
    for root, dirs, files in os.walk('.'):
        # Skip target and node_modules
        dirs[:] = [d for d in dirs if d not in ['target', 'node_modules', '.git']]
        
        for file in files:
            filepath = Path(root) / file
            
            try:
                with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
                    lines = len(f.readlines())
                    
                stats['total_files'] += 1
                stats['total_lines'] += lines
                
                ext = filepath.suffix or 'no_extension'
                stats['by_extension'][ext] += 1
                
                # Track by top-level directory
                parts = filepath.parts
                if len(parts) > 1:
                    stats['by_directory'][parts[1]] += 1
                
                # Track large files
                stats['largest_files'].append((str(filepath), lines))
                
            except Exception as e:
                print(f"Error reading {filepath}: {e}")
    
    # Sort largest files
    stats['largest_files'].sort(key=lambda x: x[1], reverse=True)
    stats['largest_files'] = stats['largest_files'][:10]
    
    return stats

def print_report(stats):
    """Print analysis report"""
    print("=" * 60)
    print("BeeBotOS Project Analysis")
    print("=" * 60)
    print()
    
    print(f"Total Files: {stats['total_files']}")
    print(f"Total Lines: {stats['total_lines']:,}")
    print()
    
    print("Files by Extension:")
    for ext, count in sorted(stats['by_extension'].items(), key=lambda x: -x[1])[:10]:
        print(f"  {ext:15} {count:5} files")
    print()
    
    print("Files by Directory:")
    for dir, count in sorted(stats['by_directory'].items(), key=lambda x: -x[1]):
        print(f"  {dir:20} {count:5} files")
    print()
    
    print("Largest Files:")
    for filepath, lines in stats['largest_files']:
        print(f"  {lines:6} lines  {filepath}")
    print()

if __name__ == '__main__':
    stats = analyze_project()
    print_report(stats)
    
    # Save JSON report
    with open('analysis_report.json', 'w') as f:
        json.dump(dict(stats), f, indent=2, default=list)
    print("Report saved to analysis_report.json")
