# Skill: Content Creator

## Description

Autonomous content creation agent for blogs, social media, newsletters, and marketing materials.

## Version

- **Version**: 1.0.0
- **Updated**: 2025-03-10

## Capabilities

### Core Functions

```yaml
functions:
  - name: generate_article
    description: Generate long-form article
    inputs:
      - topic: Article topic
      - style: Writing style
      - length: Target word count
      - keywords: SEO keywords
    outputs:
      - article: Generated article
      - meta_description: SEO meta description

  - name: create_social_post
    description: Create social media post
    inputs:
      - platform: Target platform
      - content_type: Post type
      - topic: Post topic
      - tone: Post tone
    outputs:
      - post: Generated post
      - hashtags: Suggested hashtags
      - best_time: Optimal posting time

  - name: write_newsletter
    description: Write newsletter content
    inputs:
      - sections: Newsletter sections
      - audience: Target audience
      - cta: Call-to-action
    outputs:
      - newsletter: Complete newsletter

  - name: generate_script
    description: Generate video/podcast script
    inputs:
      - format: Script format
      - duration: Target duration
      - topic: Content topic
    outputs:
      - script: Generated script
      - scene_breakdown: Scene/timing breakdown

  - name: optimize_content
    description: Optimize existing content
    inputs:
      - content: Original content
      - goal: Optimization goal
    outputs:
      - optimized: Optimized content
      - changes: List of changes made
```

## Content Types

### Blog Posts

```yaml
blog:
  formats:
    - how_to_guide
    - listicle
    - case_study
    - opinion_piece
    - interview
    - product_review
  
  seo_optimization:
    keyword_research: true
    meta_tags: true
    internal_linking: true
    readability_score: target_60_70
```

### Social Media

```yaml
social_media:
  platforms:
    twitter:
      max_length: 280
      thread_support: true
      hashtag_limit: 3
    
    linkedin:
      max_length: 3000
      professional_tone: true
      article_support: true
    
    instagram:
      caption_max: 2200
      hashtag_limit: 30
      visual_focus: true
    
    tiktok:
      script_timing: true
      trend_integration: true
      hook_optimization: true
```

### Email Marketing

```yaml
email:
  types:
    - welcome_series
    - newsletter
    - promotional
    - abandoned_cart
    - re_engagement
  
  optimization:
    subject_line_testing: true
    send_time_optimization: true
    personalization: true
    a_b_testing: true
```

## Style Templates

### Brand Voice

```yaml
brand_voice:
  dimensions:
    - formal_vs_casual: 0.6
    - serious_vs_playful: 0.4
    - professional_vs_personal: 0.7
    - informative_vs_entertaining: 0.8
  
  vocabulary:
    preferred_terms: []
    avoid_terms: []
    industry_jargon: allowed_with_explanation
```

### Content Tones

```yaml
tones:
  professional:
    characteristics: [authoritative, clear, concise]
    use_case: b2b_content
  
  conversational:
    characteristics: [friendly, approachable, engaging]
    use_case: social_media
  
  persuasive:
    characteristics: [compelling, benefit_focused, urgent]
    use_case: sales_copy
  
  educational:
    characteristics: [informative, structured, supportive]
    use_case: tutorials
```

## Content Strategy

### Editorial Calendar

```yaml
editorial_calendar:
  planning_horizon: 30  # days
  content_mix:
    educational: 0.4
    entertaining: 0.3
    promotional: 0.2
    engagement: 0.1
  
  timing:
    peak_engagement_days: [tuesday, wednesday, thursday]
    best_posting_times:
      morning: "08:00"
      lunch: "12:00"
      evening: "18:00"
```

### Trend Integration

```yaml
trends:
  monitoring_sources:
    - google_trends
    - twitter_trending
    - reddit_hot
    - industry_news
  
  integration_strategy:
    timeliness: within_24_hours
    relevance_threshold: 0.7
    brand_alignment_check: true
```

## Quality Assurance

### Checks

```yaml
qa:
  plagiarism_check: true
  grammar_check: true
  readability_analysis: true
  fact_verification: true
  brand_guideline_compliance: true
  
  approval_workflow:
    auto_publish: false
    review_required: true
    stakeholder_approval: optional
```

### Metrics

```yaml
metrics:
  engagement:
    - views
    - likes
    - shares
    - comments
    - click_through_rate
  
  content_quality:
    - readability_score
    - sentiment_analysis
    - topic_relevance
    - originality_score
```

## Usage Examples

### Generate Article

```yaml
action: generate_article
parameters:
  topic: "Web4.0 Autonomous Agents"
  style: educational
  length: 1500
  keywords: ["web4", "ai agents", "autonomous systems"]
```

### Create Social Post

```yaml
action: create_social_post
parameters:
  platform: twitter
  content_type: announcement
  topic: "BeeBotOS Launch"
  tone: exciting
```

### Write Newsletter

```yaml
action: write_newsletter
parameters:
  sections:
    - industry_news
    - product_updates
    - community_spotlight
  audience: developers
  cta: "Try BeeBotOS today"
```
