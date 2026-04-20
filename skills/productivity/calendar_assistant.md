# Calendar Assistant

## Overview

Intelligent calendar management and scheduling assistant.

## Capabilities

- Schedule optimization
- Meeting coordination
- Conflict resolution
- Time zone handling
- Reminder management

## Configuration

```yaml
name: calendar_assistant
version: 1.0.0
category: productivity
integrations:
  - google_calendar
  - outlook
  - calendly
```

## Functions

### schedule_meeting

Find optimal meeting time.

```yaml
name: schedule_meeting
input:
  title: string
  participants: array<Contact>
  duration_minutes: number
  preferred_times: optional<array<TimeRange>>
  constraints:
    time_zone: string
    avoid_weekends: boolean
    buffer_minutes: number
output:
  suggested_slots: array<TimeSlot>
  conflicts: array<Conflict>
```

### analyze_calendar

Analyze calendar for insights.

```yaml
name: analyze_calendar
input:
  period: enum[day, week, month]
output:
  metrics:
    total_meetings: number
    focus_time_hours: number
    meeting_load_percentage: number
  recommendations: array<string>
```

## Examples

Input: "Schedule a 1-hour team sync with 5 people across US, Europe, and Asia"

Output:
```
Optimal meeting times (considering all time zones):

Option 1: Tuesday 9:00 AM EST
- US East Coast: 9:00 AM
- Europe: 3:00 PM CET
- Asia: 10:00 PM JST (may be late)

Option 2: Tuesday 8:00 AM PST
- US West Coast: 8:00 AM
- US East Coast: 11:00 AM
- Europe: 5:00 PM CET
- Asia: 12:00 AM JST (midnight)

Recommendation: Option 1 with recording for Asia team members
```
