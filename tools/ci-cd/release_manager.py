#!/usr/bin/env python3
"""
BeeBotOS Release Manager
Manages versioning, changelogs, and release artifacts.
"""

import os
import re
import json
import subprocess
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Optional
from dataclasses import dataclass


@dataclass
class Version:
    major: int
    minor: int
    patch: int
    prerelease: Optional[str] = None

    def __str__(self):
        version = f"{self.major}.{self.minor}.{self.patch}"
        if self.prerelease:
            version += f"-{self.prerelease}"
        return version

    @classmethod
    def parse(cls, version_str: str) -> "Version":
        match = re.match(r"(\d+)\.(\d+)\.(\d+)(?:-(.+))?", version_str)
        if not match:
            raise ValueError(f"Invalid version string: {version_str}")
        
        major, minor, patch, prerelease = match.groups()
        return cls(
            major=int(major),
            minor=int(minor),
            patch=int(patch),
            prerelease=prerelease
        )

    def bump_major(self) -> "Version":
        return Version(self.major + 1, 0, 0)

    def bump_minor(self) -> "Version":
        return Version(self.major, self.minor + 1, 0)

    def bump_patch(self) -> "Version":
        return Version(self.major, self.minor, self.patch + 1)


class ReleaseManager:
    """Manages BeeBotOS releases."""
    
    def __init__(self, project_root: str = "."):
        self.project_root = Path(project_root)
        self.version_file = self.project_root / "VERSION"
        self.changelog_file = self.project_root / "CHANGELOG.md"
        self.package_json = self.project_root / "package.json"
        self.cargo_toml = self.project_root / "Cargo.toml"
    
    def get_current_version(self) -> Version:
        """Get current version from VERSION file."""
        if self.version_file.exists():
            version_str = self.version_file.read_text().strip()
            return Version.parse(version_str)
        
        # Try package.json
        if self.package_json.exists():
            with open(self.package_json) as f:
                data = json.load(f)
                return Version.parse(data.get("version", "0.0.0"))
        
        # Try Cargo.toml
        if self.cargo_toml.exists():
            content = self.cargo_toml.read_text()
            match = re.search(r'version = "([^"]+)"', content)
            if match:
                return Version.parse(match.group(1))
        
        return Version(0, 0, 0)
    
    def bump_version(self, bump_type: str = "patch") -> Version:
        """Bump version."""
        current = self.get_current_version()
        
        if bump_type == "major":
            new_version = current.bump_major()
        elif bump_type == "minor":
            new_version = current.bump_minor()
        else:
            new_version = current.bump_patch()
        
        # Update VERSION file
        self.version_file.write_text(str(new_version))
        
        # Update package.json if exists
        if self.package_json.exists():
            with open(self.package_json) as f:
                data = json.load(f)
            data["version"] = str(new_version)
            with open(self.package_json, "w") as f:
                json.dump(data, f, indent=2)
        
        # Update Cargo.toml if exists
        if self.cargo_toml.exists():
            content = self.cargo_toml.read_text()
            new_content = re.sub(
                r'^version = "[^"]+"',
                f'version = "{new_version}"',
                content,
                flags=re.MULTILINE
            )
            self.cargo_toml.write_text(new_content)
        
        print(f"Version bumped: {current} -> {new_version}")
        return new_version
    
    def generate_changelog(self, version: Version) -> str:
        """Generate changelog entry."""
        # Get git log since last tag
        try:
            last_tag = subprocess.check_output(
                ["git", "describe", "--tags", "--abbrev=0"],
                cwd=self.project_root,
                stderr=subprocess.DEVNULL
            ).decode().strip()
        except subprocess.CalledProcessError:
            last_tag = ""
        
        # Get commits
        if last_tag:
            commits = subprocess.check_output(
                ["git", "log", f"{last_tag}..HEAD", "--pretty=format:%s"],
                cwd=self.project_root
            ).decode().strip()
        else:
            commits = subprocess.check_output(
                ["git", "log", "--pretty=format:%s", "-20"],
                cwd=self.project_root
            ).decode().strip()
        
        # Categorize commits
        features = []
        fixes = []
        other = []
        
        for commit in commits.split("\n"):
            commit = commit.strip()
            if not commit:
                continue
            
            if commit.startswith("feat:") or commit.startswith("feature:"):
                features.append(commit.replace("feat:", "").replace("feature:", "").strip())
            elif commit.startswith("fix:"):
                fixes.append(commit.replace("fix:", "").strip())
            else:
                other.append(commit)
        
        # Generate entry
        date_str = datetime.now().strftime("%Y-%m-%d")
        entry = f"## [{version}] - {date_str}\n\n"
        
        if features:
            entry += "### Features\n"
            for f in features:
                entry += f"- {f}\n"
            entry += "\n"
        
        if fixes:
            entry += "### Bug Fixes\n"
            for f in fixes:
                entry += f"- {f}\n"
            entry += "\n"
        
        if other:
            entry += "### Other\n"
            for o in other:
                entry += f"- {o}\n"
            entry += "\n"
        
        return entry
    
    def update_changelog(self, version: Version):
        """Update CHANGELOG.md file."""
        entry = self.generate_changelog(version)
        
        if self.changelog_file.exists():
            existing = self.changelog_file.read_text()
            # Insert after header
            lines = existing.split("\n")
            header_lines = []
            rest_lines = []
            found_header = False
            
            for line in lines:
                if not found_header and line.startswith("#"):
                    header_lines.append(line)
                elif found_header or not line.startswith("#"):
                    found_header = True
                    rest_lines.append(line)
            
            new_content = "\n".join(header_lines) + "\n\n" + entry + "\n".join(rest_lines)
        else:
            new_content = f"# Changelog\n\n{entry}"
        
        self.changelog_file.write_text(new_content)
        print(f"Updated {self.changelog_file}")
    
    def create_tag(self, version: Version):
        """Create git tag."""
        tag_name = f"v{version}"
        subprocess.run(
            ["git", "tag", "-a", tag_name, "-m", f"Release {tag_name}"],
            cwd=self.project_root,
            check=True
        )
        print(f"Created tag: {tag_name}")
    
    def build_artifacts(self):
        """Build release artifacts."""
        print("Building release artifacts...")
        
        # Build Rust binaries
        if self.cargo_toml.exists():
            subprocess.run(
                ["cargo", "build", "--release"],
                cwd=self.project_root,
                check=True
            )
            print("Built Rust binaries")
        
        # Build contracts
        contracts_dir = self.project_root / "contracts"
        if contracts_dir.exists():
            print("Compiling contracts...")
            # Add contract compilation logic here
    
    def release(self, bump_type: str = "patch", skip_build: bool = False):
        """Perform full release."""
        print(f"Starting release process (bump: {bump_type})")
        
        # Bump version
        new_version = self.bump_version(bump_type)
        
        # Update changelog
        self.update_changelog(new_version)
        
        # Build artifacts
        if not skip_build:
            self.build_artifacts()
        
        # Commit changes
        subprocess.run(
            ["git", "add", "-A"],
            cwd=self.project_root,
            check=True
        )
        subprocess.run(
            ["git", "commit", "-m", f"chore(release): prepare for v{new_version}"],
            cwd=self.project_root,
            check=True
        )
        
        # Create tag
        self.create_tag(new_version)
        
        print(f"\n✅ Release v{new_version} completed!")
        print(f"   Run 'git push origin v{new_version}' to push the tag")


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description="BeeBotOS Release Manager")
    parser.add_argument("--project-root", default=".", help="Project root directory")
    
    subparsers = parser.add_subparsers(dest="command")
    
    version_parser = subparsers.add_parser("version", help="Show current version")
    
    bump_parser = subparsers.add_parser("bump", help="Bump version")
    bump_parser.add_argument("type", choices=["major", "minor", "patch"], default="patch")
    
    release_parser = subparsers.add_parser("release", help="Create release")
    release_parser.add_argument("--type", choices=["major", "minor", "patch"], default="patch")
    release_parser.add_argument("--skip-build", action="store_true")
    
    changelog_parser = subparsers.add_parser("changelog", help="Generate changelog")
    
    args = parser.parse_args()
    
    manager = ReleaseManager(args.project_root)
    
    if args.command == "version":
        version = manager.get_current_version()
        print(f"Current version: {version}")
    elif args.command == "bump":
        manager.bump_version(args.type)
    elif args.command == "release":
        manager.release(args.type, args.skip_build)
    elif args.command == "changelog":
        version = manager.get_current_version()
        changelog = manager.generate_changelog(version)
        print(changelog)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
