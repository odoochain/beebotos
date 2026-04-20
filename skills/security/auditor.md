# Security Auditor

## Description

Autonomous security auditing agent for smart contracts, code analysis, and vulnerability detection.

## Version

- **Version**: 1.0.0
- **Updated**: 2025-03-10

## Capabilities

### Core Functions

```yaml
functions:
  - name: audit_contract
    description: Perform comprehensive smart contract audit
    inputs:
      - contract_address: Contract address
      - source_code: Contract source code
      - chain: Blockchain network
      - audit_depth: quick | standard | deep
    outputs:
      - audit_report: Detailed audit findings
      - risk_score: Overall risk assessment
      - recommendations: Remediation suggestions

  - name: analyze_code
    description: Static analysis of source code
    inputs:
      - code: Source code to analyze
      - language: Programming language
      - analysis_type: security | performance | style
    outputs:
      - issues: List of identified issues
      - metrics: Code quality metrics

  - name: detect_vulnerabilities
    description: Detect known vulnerability patterns
    inputs:
      - target: Target to scan
      - vulnerability_db: Vulnerability database to use
    outputs:
      - vulnerabilities: Detected vulnerabilities
      - severity_distribution: Breakdown by severity

  - name: fuzz_test
    description: Fuzz testing for edge cases
    inputs:
      - target: Target contract/function
      - duration: Fuzzing duration
      - corpus: Initial test corpus
    outputs:
      - findings: Discovered edge cases
      - coverage: Code coverage achieved

  - name: generate_exploit
    description: Generate proof-of-concept exploit
    inputs:
      - vulnerability: Vulnerability details
      - target_environment: Test environment
    outputs:
      - exploit_code: PoC exploit code
      - remediation: Fix suggestion
```

## Vulnerability Categories

```yaml
vulnerabilities:
  smart_contracts:
    - reentrancy
    - integer_overflow
    - access_control
    - front_running
    - timestamp_dependence
    - unchecked_calls
    - delegatecall_injection
    - storage_collision
    - randomness_manipulation
  
  general_security:
    - injection_attacks
    - authentication_flaws
    - sensitive_data_exposure
    - xml_external_entities
    - broken_access_control
    - security_misconfiguration
    - cross_site_scripting
    - insecure_deserialization
    - vulnerable_components
```

## Analysis Engines

```yaml
engines:
  static_analysis:
    - slither
    - mythril
    - semgrep
    - echidna
    - custom_rules
  
  dynamic_analysis:
    - fuzzing
    - symbolic_execution
    - taint_analysis
  
  manual_review:
    - pattern_matching
    - business_logic_review
    - economic_security
```

## Risk Scoring

```yaml
risk_matrix:
  severity:
    critical: 9-10
    high: 7-8
    medium: 4-6
    low: 1-3
    informational: 0
  
  likelihood:
    almost_certain: 5
    likely: 4
    possible: 3
    unlikely: 2
    rare: 1
  
  scoring_formula: severity * likelihood
  
  thresholds:
    block_deployment: 40
    high_risk: 28
    medium_risk: 12
    acceptable: < 12
```

## Report Generation

```yaml
report_sections:
  - executive_summary
  - methodology
  - findings:
      - vulnerability_details
      - proof_of_concept
      - risk_assessment
      - remediation
  - code_quality_metrics
  - compliance_check
  - appendices

templates:
  - comprehensive
  - executive
  - technical
  - developer
```

## Configuration

```yaml
config:
  default_audit_depth: standard
  max_analysis_time: 3600  # seconds
  parallel_analysis: true
  
  severity_filters:
    min_severity: low
    include_informational: true
  
  false_positive_handling:
    suppression_comments: true
    learning_mode: true
  
  integrations:
    - github
    - gitlab
    - ci_cd_pipelines
    - issue_trackers
```

## Usage Examples

### Audit Contract

```yaml
action: audit_contract
parameters:
  contract_address: "0x..."
  chain: monad
  audit_depth: deep
```

### Code Analysis

```yaml
action: analyze_code
parameters:
  code: "contract source"
  language: solidity
  analysis_type: security
```

### Fuzz Test

```yaml
action: fuzz_test
parameters:
  target: "0x..."
  duration: 3600
  corpus: []
```
