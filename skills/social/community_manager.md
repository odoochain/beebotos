# Community Manager

## Description

Autonomous community management agent for Discord, forums, and social platforms with moderation and engagement capabilities.

## Version

- **Version**: 1.0.0
- **Updated**: 2025-03-10

## Capabilities

### Core Functions

```yaml
functions:
  - name: moderate_content
    description: Moderate community content
    inputs:
      - content: Content to moderate
      - platform: Platform type
      - rules: Community rules
    outputs:
      - decision: moderation decision
      - confidence: confidence score
      - action: recommended action

  - name: respond_to_question
    description: Answer community questions
    inputs:
      - question: User question
      - context: Conversation context
      - knowledge_base: Reference docs
    outputs:
      - answer: Generated answer
      - sources: Reference sources

  - name: initiate_engagement
    description: Start engagement activities
    inputs:
      - activity_type: Type of engagement
      - target_audience: Target members
    outputs:
      - activity: Engagement activity details

  - name: analyze_sentiment
    description: Analyze community sentiment
    inputs:
      - timeframe: Analysis period
      - channels: Channels to analyze
    outputs:
      - sentiment_report: Sentiment analysis

  - name: identify_advocates
    description: Identify community advocates
    inputs:
      - criteria: Advocate criteria
      - min_activity: Minimum activity level
    outputs:
      - advocates: List of potential advocates

  - name: handle_dispute
    description: Mediate community disputes
    inputs:
      - parties: Involved parties
      - context: Dispute context
    outputs:
      - resolution: Proposed resolution
```

## Moderation

### Content Policies

```yaml
content_policies:
  prohibited_content:
    - hate_speech
    - harassment
    - spam
    - scams
    - nsfw
    - doxxing
  
  restricted_content:
    - self_promotion:
        allowed: true
        limits: weekly_promo_channel_only
    - external_links:
        allowed: true
        verification: auto_check
  
  enforcement:
    first_violation: warning
    second_violation: timeout_1h
    third_violation: timeout_24h
    severe_violation: immediate_ban
```

### Auto-Moderation

```yaml
auto_moderation:
  spam_detection:
    identical_message_threshold: 3
    time_window_minutes: 5
    action: mute_temporarily
  
  raid_protection:
    join_rate_monitoring: true
    mass_mention_detection: true
    suspicious_link_detection: true
  
  content_filtering:
    profanity_filter: strict
    link_filter: smart
    image_moderation: ai_assisted
```

## Engagement

### Activities

```yaml
engagement_activities:
  regular:
    - daily_question
    - weekly_highlight
    - monthly_recap
    - spotlights
  
  events:
    - ama_sessions
    - hackathons
    - contests
    - giveaways
  
  recognition:
    - contributor_badges
    - helpful_member_awards
    - milestone_celebrations
```

### Conversation Starters

```yaml
conversation_starters:
  frequency: 3  # per day
  topics:
    - industry_news
    - product_updates
    - community_showcase
    - educational_content
    - fun_and_games
  
  timing:
    morning: "09:00"
    afternoon: "14:00"
    evening: "19:00"
```

## Support

### FAQ Automation

```yaml
faq_automation:
  response_time_target: 30  # seconds
  confidence_threshold: 0.85
  escalation_threshold: 0.6
  
  knowledge_sources:
    - documentation
    - previous_tickets
    - community_wiki
    - product_changelog
  
  fallback:
    action: create_support_ticket
    notify: support_team
```

### Ticket Management

```yaml
ticket_management:
  categorization:
    - technical_issue
    - billing_question
    - feature_request
    - account_help
    - general_inquiry
  
  prioritization:
    critical: response_1h
    high: response_4h
    medium: response_24h
    low: response_72h
  
  routing:
    auto_assign: true
    skill_based: true
    load_balanced: true
```

## Analytics

### Metrics

```yaml
metrics:
  growth:
    - new_members
    - retention_rate
    - activation_rate
    - churn_rate
  
  engagement:
    - daily_active_users
    - messages_per_user
    - reaction_rate
    - thread_participation
  
  health:
    - sentiment_score
    - support_ticket_volume
    - moderation_actions
    - advocate_activity
```

### Reporting

```yaml
reporting:
  daily:
    - activity_summary
    - moderation_log
    - support_queue_status
  
  weekly:
    - growth_report
    - engagement_analysis
    - content_performance
  
  monthly:
    - comprehensive_dashboard
    - trend_analysis
    - recommendations
```

## Integration

```yaml
integrations:
  platforms:
    - discord
    - telegram
    - discourse
    - reddit
    - twitter
  
  tools:
    - crm: hubspot
    - analytics: amplitude
    - support: zendesk
    - notifications: slack
```

## Usage Examples

### Moderate Content

```yaml
action: moderate_content
parameters:
  content: "User submitted message"
  platform: discord
  rules: community_guidelines_v2
```

### Answer Question

```yaml
action: respond_to_question
parameters:
  question: "How do I stake BEE tokens?"
  context: general_chat
  knowledge_base: beebotos_docs
```

### Analyze Sentiment

```yaml
action: analyze_sentiment
parameters:
  timeframe: 7d
  channels: [general, support, announcements]
```
