#!/usr/bin/env python3
"""
BeeBotOS Backup Manager
Handles automated backups of agent data and configurations.
"""

import os
import sys
import json
import gzip
import shutil
import hashlib
from datetime import datetime, timedelta
from pathlib import Path
from typing import List, Dict, Optional


class BackupManager:
    """Manages BeeBotOS backups."""
    
    def __init__(self, backup_dir: str = "data/backups"):
        self.backup_dir = Path(backup_dir)
        self.backup_dir.mkdir(parents=True, exist_ok=True)

        self.sources = [
            "data",
            "config",
        ]
        
    def create_backup(self, name: Optional[str] = None) -> str:
        """Create a new backup."""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        backup_name = name or f"backup_{timestamp}"
        backup_path = self.backup_dir / backup_name
        
        print(f"Creating backup: {backup_name}")
        
        backup_path.mkdir(exist_ok=True)
        
        manifest = {
            "name": backup_name,
            "created_at": timestamp,
            "sources": [],
        }
        
        for source in self.sources:
            if os.path.exists(source):
                dest = backup_path / Path(source).name
                self._backup_path(source, dest)
                
                manifest["sources"].append({
                    "path": source,
                    "backup_path": str(dest),
                    "checksum": self._calculate_checksum(dest),
                })
        
        manifest_path = backup_path / "manifest.json"
        with open(manifest_path, "w") as f:
            json.dump(manifest, f, indent=2)
        
        print(f"Backup completed: {backup_path}")
        return str(backup_path)
    
    def _backup_path(self, source: str, dest: Path):
        """Backup a single path."""
        if os.path.isdir(source):
            shutil.copytree(source, dest, dirs_exist_ok=True)
        else:
            shutil.copy2(source, dest)
    
    def _calculate_checksum(self, path: Path) -> str:
        """Calculate directory checksum."""
        hasher = hashlib.sha256()
        
        if path.is_file():
            with open(path, "rb") as f:
                hasher.update(f.read())
        elif path.is_dir():
            for file in sorted(path.rglob("*")):
                if file.is_file():
                    with open(file, "rb") as f:
                        hasher.update(f.read())
        
        return hasher.hexdigest()
    
    def list_backups(self) -> List[Dict]:
        """List all backups."""
        backups = []
        
        for item in self.backup_dir.iterdir():
            if item.is_dir():
                manifest_path = item / "manifest.json"
                if manifest_path.exists():
                    with open(manifest_path) as f:
                        manifest = json.load(f)
                        manifest["size"] = self._get_dir_size(item)
                        backups.append(manifest)
        
        return sorted(backups, key=lambda x: x["created_at"], reverse=True)
    
    def _get_dir_size(self, path: Path) -> int:
        """Get directory size in bytes."""
        total = 0
        for item in path.rglob("*"):
            if item.is_file():
                total += item.stat().st_size
        return total
    
    def restore_backup(self, backup_name: str, target_dir: Optional[str] = None):
        """Restore a backup."""
        backup_path = self.backup_dir / backup_name
        
        if not backup_path.exists():
            raise ValueError(f"Backup not found: {backup_name}")
        
        manifest_path = backup_path / "manifest.json"
        with open(manifest_path) as f:
            manifest = json.load(f)
        
        print(f"Restoring backup: {backup_name}")
        
        for source_info in manifest["sources"]:
            source_path = Path(source_info["backup_path"])
            dest_path = target_dir or source_info["path"]
            
            if os.path.exists(dest_path):
                backup_suffix = datetime.now().strftime("%Y%m%d_%H%M%S")
                shutil.move(dest_path, f"{dest_path}.bak.{backup_suffix}")
            
            if source_path.is_dir():
                shutil.copytree(source_path, dest_path, dirs_exist_ok=True)
            else:
                shutil.copy2(source_path, dest_path)
        
        print(f"Restore completed: {backup_name}")
    
    def cleanup_old_backups(self, keep_days: int = 30):
        """Remove backups older than specified days."""
        cutoff = datetime.now() - timedelta(days=keep_days)
        
        for item in self.backup_dir.iterdir():
            if item.is_dir():
                manifest_path = item / "manifest.json"
                if manifest_path.exists():
                    with open(manifest_path) as f:
                        manifest = json.load(f)
                    
                    created = datetime.strptime(manifest["created_at"], "%Y%m%d_%H%M%S")
                    if created < cutoff:
                        print(f"Removing old backup: {item.name}")
                        shutil.rmtree(item)


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description="BeeBotOS Backup Manager")
    parser.add_argument("--backup-dir", default="data/backups")
    
    subparsers = parser.add_subparsers(dest="command")
    
    create_parser = subparsers.add_parser("create", help="Create a backup")
    create_parser.add_argument("--name", help="Backup name")
    
    list_parser = subparsers.add_parser("list", help="List backups")
    
    restore_parser = subparsers.add_parser("restore", help="Restore a backup")
    restore_parser.add_argument("name", help="Backup name to restore")
    restore_parser.add_argument("--target", help="Target directory")
    
    cleanup_parser = subparsers.add_parser("cleanup", help="Clean old backups")
    cleanup_parser.add_argument("--keep-days", type=int, default=30)
    
    args = parser.parse_args()
    
    manager = BackupManager(args.backup_dir)
    
    if args.command == "create":
        manager.create_backup(args.name)
    elif args.command == "list":
        backups = manager.list_backups()
        for backup in backups:
            size_mb = backup["size"] / (1024 * 1024)
            print(f"{backup['name']:30} | {backup['created_at']} | {size_mb:.1f} MB")
    elif args.command == "restore":
        manager.restore_backup(args.name, args.target)
    elif args.command == "cleanup":
        manager.cleanup_old_backups(args.keep_days)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
