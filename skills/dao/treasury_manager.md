# Treasury Manager Skill

## Overview

Manages DAO treasury operations and financial planning.

## Capabilities

- Budget allocation
- Financial reporting
- Risk management
- Investment strategy
- Cash flow planning

## Configuration

```yaml
name: treasury_manager
version: 1.0.0
category: dao
```

## Functions

### analyze_treasury

Analyze treasury health.

```yaml
name: analyze_treasury
input:
  treasury_address: string
output:
  total_value: AssetBreakdown
  runway_months: number
  risk_metrics: RiskMetrics
  recommendations: array<string>
```

### create_budget

Create quarterly budget.

```yaml
name: create_budget
input:
  quarter: string
  categories: array<Category>
  total_allocation: number
output:
  budget: Budget
  projections: Projections
```

## Example

Input: "Analyze treasury for Q2 2026 planning"

Output:
```json
{
  "treasury_analysis": {
    "total_value_usd": "$50,000,000",
    "assets": {
      "BEE": "60%",
      "ETH": "20%",
      "USDC": "15%",
      "Other": "5%"
    },
    "monthly_burn_rate": "$800,000",
    "runway": "52 months"
  },
  "recommendations": [
    "Diversify 10% of BEE holdings to stablecoins",
    "Establish 12-month operating reserve",
    "Set up streaming payments for recurring expenses"
  ],
  "q2_budget_proposal": {
    "development": "1,200,000 BEE",
    "marketing": "800,000 BEE",
    "operations": "400,000 BEE",
    "grants": "600,000 BEE"
  }
}
```
