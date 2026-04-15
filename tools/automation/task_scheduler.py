#!/usr/bin/env python3
"""
BeeBotOS Task Scheduler
Automates task scheduling and execution for agents.
"""

import json
import time
import schedule
import threading
from datetime import datetime, timedelta
from typing import Dict, List, Callable, Optional
from dataclasses import dataclass, asdict


@dataclass
class ScheduledTask:
    task_id: str
    name: str
    cron_expression: str
    command: str
    enabled: bool
    last_run: Optional[str]
    next_run: Optional[str]
    run_count: int
    max_retries: int


class TaskScheduler:
    """Schedules and manages automated tasks."""
    
    def __init__(self, config_file: str = "data/scheduler.json"):
        self.config_file = config_file
        self.tasks: Dict[str, ScheduledTask] = {}
        self.handlers: Dict[str, Callable] = {}
        self.running = False
        self.thread: Optional[threading.Thread] = None
        
    def load_tasks(self):
        """Load scheduled tasks from config."""
        try:
            with open(self.config_file) as f:
                data = json.load(f)
                for task_data in data.get("tasks", []):
                    task = ScheduledTask(**task_data)
                    self.tasks[task.task_id] = task
        except FileNotFoundError:
            pass
    
    def save_tasks(self):
        """Save scheduled tasks to config."""
        data = {
            "tasks": [asdict(t) for t in self.tasks.values()]
        }
        with open(self.config_file, "w") as f:
            json.dump(data, f, indent=2)
    
    def register_handler(self, command: str, handler: Callable):
        """Register a command handler."""
        self.handlers[command] = handler
    
    def add_task(self, name: str, cron: str, command: str, max_retries: int = 3) -> str:
        """Add a new scheduled task."""
        task_id = f"task_{int(time.time())}_{len(self.tasks)}"
        
        task = ScheduledTask(
            task_id=task_id,
            name=name,
            cron_expression=cron,
            command=command,
            enabled=True,
            last_run=None,
            next_run=None,
            run_count=0,
            max_retries=max_retries
        )
        
        self.tasks[task_id] = task
        self._schedule_task(task)
        self.save_tasks()
        
        return task_id
    
    def _schedule_task(self, task: ScheduledTask):
        """Schedule a task using the schedule library."""
        if not task.enabled:
            return
        
        # Parse cron expression (simplified: minute hour)
        parts = task.cron_expression.split()
        if len(parts) == 2:
            minute, hour = parts
            
            if minute == "*" and hour == "*":
                schedule.every().minute.do(self._execute_task, task.task_id)
            elif minute == "*":
                schedule.every().hour.at(f":{hour}").do(self._execute_task, task.task_id)
            else:
                schedule.every().day.at(f"{hour}:{minute}").do(self._execute_task, task.task_id)
    
    def _execute_task(self, task_id: str):
        """Execute a scheduled task."""
        task = self.tasks.get(task_id)
        if not task or not task.enabled:
            return
        
        print(f"[{datetime.now()}] Executing task: {task.name}")
        
        # Extract command and arguments
        parts = task.command.split()
        command = parts[0]
        args = parts[1:] if len(parts) > 1 else []
        
        if command in self.handlers:
            try:
                self.handlers[command](*args)
                task.run_count += 1
                task.last_run = datetime.now().isoformat()
            except Exception as e:
                print(f"Task {task.name} failed: {e}")
        else:
            print(f"No handler for command: {command}")
        
        self.save_tasks()
    
    def start(self):
        """Start the scheduler."""
        self.running = True
        
        # Schedule all enabled tasks
        for task in self.tasks.values():
            self._schedule_task(task)
        
        def run_scheduler():
            while self.running:
                schedule.run_pending()
                time.sleep(1)
        
        self.thread = threading.Thread(target=run_scheduler)
        self.thread.start()
        print("Task scheduler started")
    
    def stop(self):
        """Stop the scheduler."""
        self.running = False
        if self.thread:
            self.thread.join()
        print("Task scheduler stopped")
    
    def list_tasks(self) -> List[ScheduledTask]:
        """List all scheduled tasks."""
        return list(self.tasks.values())
    
    def enable_task(self, task_id: str):
        """Enable a task."""
        if task_id in self.tasks:
            self.tasks[task_id].enabled = True
            self._schedule_task(self.tasks[task_id])
            self.save_tasks()
    
    def disable_task(self, task_id: str):
        """Disable a task."""
        if task_id in self.tasks:
            self.tasks[task_id].enabled = False
            schedule.clear()
            # Re-schedule all enabled tasks
            for task in self.tasks.values():
                self._schedule_task(task)
            self.save_tasks()
    
    def remove_task(self, task_id: str):
        """Remove a task."""
        if task_id in self.tasks:
            del self.tasks[task_id]
            schedule.clear()
            for task in self.tasks.values():
                self._schedule_task(task)
            self.save_tasks()


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description="BeeBotOS Task Scheduler")
    parser.add_argument("--config", default="data/scheduler.json")
    
    subparsers = parser.add_subparsers(dest="command")
    
    add_parser = subparsers.add_parser("add", help="Add a task")
    add_parser.add_argument("--name", required=True)
    add_parser.add_argument("--cron", required=True, help="Cron expression (minute hour)")
    add_parser.add_argument("--cmd", required=True, help="Command to execute")
    
    list_parser = subparsers.add_parser("list", help="List tasks")
    
    start_parser = subparsers.add_parser("start", help="Start scheduler")
    
    args = parser.parse_args()
    
    scheduler = TaskScheduler(args.config)
    scheduler.load_tasks()
    
    # Register example handlers
    def backup_task():
        print("Running backup...")
    
    def cleanup_task():
        print("Running cleanup...")
    
    scheduler.register_handler("backup", backup_task)
    scheduler.register_handler("cleanup", cleanup_task)
    
    if args.command == "add":
        task_id = scheduler.add_task(args.name, args.cron, args.cmd)
        print(f"Task added: {task_id}")
    elif args.command == "list":
        tasks = scheduler.list_tasks()
        for task in tasks:
            status = "enabled" if task.enabled else "disabled"
            print(f"{task.task_id}: {task.name} ({status}) - {task.cron_expression}")
    elif args.command == "start":
        scheduler.start()
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            scheduler.stop()
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
