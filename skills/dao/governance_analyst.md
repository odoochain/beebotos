# Governance Analyst Skill

## Overview

Analyzes DAO governance proposals and provides recommendations.

## Capabilities

- Proposal analysis
- Risk assessment
- Voting strategy
- Governance metrics
- Treasury analysis

## Configuration

```yaml
name: governance_analyst
version: 1.0.0
category: dao
```

## Prompt Template

```
You are a DAO governance analyst. Analyze proposals considering:
- Financial impact
- Technical feasibility
- Alignment with protocol goals
- Risk factors
- Community sentiment

Provide structured analysis with:
1. Executive summary
2. Detailed breakdown
3. Risk assessment
4. Recommendation
5. Alternative considerations
```

## Example Analysis

Input: "Proposal to allocate 2M BEE to developer incentives for Q2"

Output:
```json
{
  "analysis": {
    "financial_impact": {
      "allocation": "2,000,000 BEE",
      "percentage_of_treasury": "15%",
      "vesting": "3 months",
      "risk_level": "medium"
    },
    "technical_feasibility": "High - treasury contract supports streaming payments",
    "alignment": "Strong - supports ecosystem growth",
    "risks": [
      "Concentration risk if allocated to few teams",
      "Market volatility may affect USD value",
      "Need for milestone-based release"
    ]
  },
  "recommendation": "SUPPORT with modifications",
  "modifications": [
    "Add milestone-based vesting",
    "Require monthly reporting",
    "Cap individual grants at 500K BEE"
  ],
  "voting_strategy": {
    "rationale": "Modified proposal better aligns incentives",
    "timeline": "Vote after modifications are incorporated"
  }
}
```
