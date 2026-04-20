# Data Analyst

## Description

Autonomous data analysis agent for querying, visualizing, and deriving insights from on-chain and off-chain data.

## Version

- **Version**: 1.0.0
- **Updated**: 2025-03-10

## Capabilities

### Core Functions

```yaml
functions:
  - name: query_data
    description: Execute data queries
    inputs:
      - data_source: Source to query
      - query: Query specification
      - time_range: Time range for data
    outputs:
      - results: Query results
      - metadata: Result metadata

  - name: analyze_trends
    description: Identify trends in data
    inputs:
      - data: Input dataset
      - trend_type: trend types to detect
    outputs:
      - trends: Identified trends
      - confidence: Confidence scores

  - name: generate_visualization
    description: Create data visualizations
    inputs:
      - data: Data to visualize
      - chart_type: Type of chart
      - styling: Visual styling options
    outputs:
      - chart: Generated chart
      - interactive_html: Interactive version

  - name: predict_metrics
    description: Predict future metrics
    inputs:
      - historical_data: Historical time series
      - forecast_horizon: Prediction horizon
      - model_type: Prediction model
    outputs:
      - predictions: Forecasted values
      - confidence_intervals: Prediction intervals

  - name: detect_anomalies
    description: Detect anomalous patterns
    inputs:
      - data: Dataset to analyze
      - sensitivity: Detection sensitivity
    outputs:
      - anomalies: Detected anomalies
      - explanations: Anomaly explanations
```

## Data Sources

```yaml
data_sources:
  on_chain:
    - monad
    - ethereum
    - arbitrum
    - polygon
    - dune_analytics
    - subgraphs
  
  off_chain:
    - exchange_apis
    - social_media
    - news_feeds
    - market_data
    - sentiment_data
```

## Analysis Types

```yaml
analysis_types:
  descriptive:
    - summary_statistics
    - distribution_analysis
    - correlation_analysis
  
  diagnostic:
    - root_cause_analysis
    - anomaly_detection
    - cohort_analysis
  
  predictive:
    - time_series_forecasting
    - regression_analysis
    - classification
  
  prescriptive:
    - optimization
    - recommendation
    - scenario_modeling
```

## Visualization Types

```yaml
visualizations:
  basic:
    - line_chart
    - bar_chart
    - pie_chart
    - scatter_plot
    - area_chart
  
  advanced:
    - candlestick
    - heatmap
    - sankey_diagram
    - network_graph
    - geographic_map
  
  on_chain:
    - transaction_flow
    - wallet_clustering
    - smart_contract_interactions
    - token_movement
```

## Machine Learning Models

```yaml
models:
  forecasting:
    - arima
    - prophet
    - lstm
    - transformer
  
  anomaly_detection:
    - isolation_forest
    - one_class_svm
    - autoencoder
    - statistical_methods
  
  clustering:
    - k_means
    - dbscan
    - hierarchical
    - gaussian_mixture
```

## Configuration

```yaml
config:
  caching:
    enabled: true
    ttl_seconds: 300
  
  query_limits:
    max_rows: 100000
    timeout_seconds: 60
  
  visualization:
    default_theme: dark
    color_palette: beebotos
    responsive: true
  
  export_formats:
    - csv
    - json
    - parquet
    - pdf
    - png
    - svg
```

## Usage Examples

### Query On-Chain Data

```yaml
action: query_data
parameters:
  data_source: monad
  query:
    type: transactions
    filters:
      from_block: 1000000
      to_block: 1000100
      address: "0x..."
```

### Analyze Token Trends

```yaml
action: analyze_trends
parameters:
  data: token_price_data
  trend_type: [seasonality, momentum, volatility]
```

### Generate Dashboard

```yaml
action: generate_visualization
parameters:
  data: defi_metrics
  chart_type: dashboard
  styling:
    theme: professional
    layout: grid
```
