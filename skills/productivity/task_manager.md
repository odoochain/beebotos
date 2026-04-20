# Task Manager

## Overview

Intelligent task management and productivity optimization.

## Capabilities

- Task decomposition
- Priority management
- Time estimation
- Scheduling optimization
- Progress tracking

## Configuration

```yaml
name: task_manager
version: 1.0.0
category: productivity
```

## Functions

### create_task

Create a new task with subtasks.

```yaml
name: create_task
input:
  title: string
  description: string
  due_date: optional<string>
  priority: optional<enum[low, medium, high, critical]>
  tags: optional<array<string>>
output:
  task_id: string
  subtasks: array<Subtask>
  estimated_hours: number
```

### analyze_schedule

Analyze and optimize schedule.

```yaml
name: analyze_schedule
input:
  tasks: array<Task>
  constraints:
    working_hours: string
    breaks: array<TimeRange>
output:
  optimized_schedule: Schedule
  conflicts: array<Conflict>
  recommendations: array<string>
```

## Examples

Input: "I need to launch a product in 2 weeks. Tasks: finalize design, develop frontend, develop backend, testing, marketing."

Output:
```json
{
  "project": "Product Launch",
  "timeline": "14 days",
  "phases": [
    {
      "phase": "Week 1",
      "tasks": [
        {
          "task": "Finalize Design",
          "days": "1-3",
          "priority": "critical",
          "blockers": []
        },
        {
          "task": "Backend Development",
          "days": "1-7",
          "priority": "high",
          "blockers": ["Finalize Design"]
        },
        {
          "task": "Frontend Development",
          "days": "4-10",
          "priority": "high",
          "blockers": ["Finalize Design"]
        }
      ]
    },
    {
      "phase": "Week 2",
      "tasks": [
        {
          "task": "Testing",
          "days": "8-12",
          "priority": "critical",
          "blockers": ["Backend Development", "Frontend Development"]
        },
        {
          "task": "Marketing Preparation",
          "days": "11-14",
          "priority": "medium",
          "blockers": []
        }
      ]
    }
  ],
  "risks": [
    "Tight timeline for testing",
    "Parallel development may cause integration issues"
  ]
}
```
