# 🧠 beebotos-social-brain 模块架构与代码逻辑分析

## 📑 目录

1. [架构概述](#架构概述)
2. [核心模块详解](#核心模块详解)
3. [数据流转分析](#数据流转分析)
4. [模块间关系](#模块间关系)
5. [函数接口体系](#函数接口体系)
6. [关键算法实现](#关键算法实现)
7. [状态管理机制](#状态管理机制)
8. [扩展与集成](#扩展与集成)

---

## 架构概述

### 1.1 系统定位

`beebotos-social-brain` 是 BeeBotOS 的**认知架构层 (Layer 2)**，位于核心层之上、应用层之下，为 Agent 提供类人的认知能力。

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│         (Agents, Skills, Workflows)                        │
├─────────────────────────────────────────────────────────────┤
│  🧠 Social Brain Layer (beebotos-social-brain)             │
│  ┌──────────┬──────────┬──────────┬──────────┐             │
│  │  NEAT    │   PAD    │  Memory  │Cognition │             │
│  │(神经进化)│ (情绪)   │ (记忆)   │ (认知)   │             │
│  ├──────────┼──────────┼──────────┼──────────┤             │
│  │Personality│Reasoning│Attention │Learning  │             │
│  │(人格)    │ (推理)   │ (注意力) │ (学习)   │             │
│  ├──────────┴──────────┴──────────┴──────────┤             │
│  │  Social │ Knowledge │ Creativity │ Language │           │
│  │ (社交)  │ (知识)    │ (创造力)   │ (语言)   │           │
│  └───────────────────────────────────────────┘             │
├─────────────────────────────────────────────────────────────┤
│                    Core Layer (beebotos-core)              │
│              (AgentId, Message, Event System)              │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 设计哲学

| 原则 | 实现方式 |
|------|---------|
| **模块化** | 17个子模块独立演化，通过 trait 解耦 |
| **可配置** | `BrainConfig` 支持轻量/标准/高性能模式 |
| **可观测** | 全程使用 `tracing` 记录状态变化 |
| **类型安全** | 强类型枚举 + Result 错误处理 |
| **性能优先** | 关键路径零分配，支持并行计算 |

### 1.3 入口架构

```rust
// lib.rs - 统一暴露接口
pub struct BrainConfig {
    pub neat: NeatConfig,           // NEAT 配置
    pub pad: PadConfig,             // 情绪配置
    pub memory: MemoryConfig,       // 记忆配置
    pub personality: PersonalityConfig, // 人格配置
    pub parallel: ParallelConfig,   // 并行配置
    pub features: FeatureToggles,   // 功能开关
}

// api.rs - 高层统一 API
pub struct SocialBrainApi {
    cognitive_state: CognitiveState,        // 认知状态
    memory: UnifiedMemory,                  // 统一记忆
    emotional_intelligence: EmotionalIntelligence, // 情绪智能
    personality: OceanProfile,              // 人格档案
    network: Option<NeuralNetwork>,         // 神经网络
    config: ApiConfig,                      // API配置
}
```

---

## 核心模块详解

### 2.1 NEAT 模块 (神经进化)

**文件位置**: `src/neat/`

#### 2.1.1 核心结构

```rust
// Genome - 基因型 (Genotype)
pub struct Genome {
    pub id: u64,
    pub layers: Vec<LayerGene>,           // 层基因
    pub connections: Vec<ConnectionGene>, // 连接基因 (关键创新)
    pub learning_params: LearningParams,
    pub fitness: f32,
    pub adjusted_fitness: f32,
    pub species_id: Option<u64>,
}

// ConnectionGene - NEAT 的核心创新
pub struct ConnectionGene {
    pub in_node: u64,
    pub out_node: u64,
    pub weight: f32,
    pub enabled: bool,
    pub innovation_number: u64,  // 历史标记，用于对齐基因
    pub is_recurrent: bool,
}

// NeuralNetwork - 表现型 (Phenotype)
pub struct NeuralNetwork {
    pub layers: Vec<Layer>,
    pub connections: Vec<Connection>,
    pub node_map: HashMap<u64, usize>, // node_id -> layer_index
}
```

#### 2.1.2 进化流程

```
初始种群创建
    ↓
适应度评估 (Fitness Evaluation)
    ↓
物种形成 (Speciation) - 基于兼容性距离
    ↓
适应度调整 (Fitness Sharing) - 同物种内竞争
    ↓
选择 (Selection) - 轮盘赌或锦标赛
    ↓
交叉 (Crossover) - 基于创新号对齐基因
    ↓
变异 (Mutation) - 权重变异 + 结构变异
    ↓
新一代种群
    ↓
重复直到收敛
```

**关键代码逻辑**:

```rust
// 兼容性距离计算 (Compatibility Distance)
pub fn compatibility_distance(&self, other: &Genome, config: &NeatConfig) -> f32 {
    let mut disjoint = 0.0;  // 不相交基因
    let mut excess = 0.0;    //  excess 基因
    let mut weight_diff = 0.0; // 权重差异
    let mut matching = 0;

    // 基于创新号对齐基因
    for i in 0..=max_innovation {
        match (c1, c2) {
            (Some(c1), Some(c2)) => {
                weight_diff += (c1.weight - c2.weight).abs();
                matching += 1;
            }
            (Some(_), None) | (None, Some(_)) => {
                // 根据位置判断是 disjoint 还是 excess
                if i < max_innovation / 2 { disjoint += 1.0; } 
                else { excess += 1.0; }
            }
            (None, None) => {}
        }
    }

    // 公式: c1*E/N + c2*D/N + c3*W̄
    (config.excess_coefficient * excess + config.disjoint_coefficient * disjoint) / n
        + config.weight_coefficient * weight_diff
}

// 结构变异 - 添加节点
fn add_node_mutation(&mut self, innovations: &mut InnovationTracker) {
    // 1. 选择随机连接
    // 2. 禁用旧连接
    // 3. 创建新节点
    // 4. 添加两个新连接 (in->new, new->out)
    // 5. 分配创新号
}

// 结构变异 - 添加连接
fn add_connection_mutation(&mut self, innovations: &mut InnovationTracker) {
    // 尝试在未连接的节点对间创建新连接
    // 避免重复连接和无效连接
}
```

#### 2.1.3 创新号追踪

```rust
pub struct InnovationTracker {
    next_innovation: usize,
    // (from_node, to_node) -> innovation_number
    connection_innovations: HashMap<(usize, usize), usize>,
    // connection_innovation -> node_id
    node_innovations: HashMap<usize, usize>,
}

// 全局历史标记确保同代相同结构获得相同创新号
pub fn get_connection_innovation(&mut self, from: usize, to: usize) -> usize {
    let key = (from, to);
    if let Some(&innovation) = self.connection_innovations.get(&key) {
        innovation  // 已存在，返回现有创新号
    } else {
        let innovation = self.next_innovation;
        self.connection_innovations.insert(key, innovation);
        self.next_innovation += 1;
        innovation  // 新创新号
    }
}
```

---

### 2.2 PAD 模块 (情绪模型)

**文件位置**: `src/pad/`

#### 2.2.1 理论基础

**PAD 情绪模型** - 三维情绪空间：
- **P**leasure (愉悦度): -1.0 (不愉快) ~ +1.0 (愉悦)
- **A**rousal (唤醒度): 0.0 (平静) ~ 1.0 (兴奋)
- **D**ominance (支配度): 0.0 (顺从) ~ 1.0 (支配)

```rust
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Pad {
    pub pleasure: f32,    // -1.0 to 1.0
    pub arousal: f32,     // 0.0 to 1.0
    pub dominance: f32,   // 0.0 to 1.0
}

// 预定义情绪状态
impl Pad {
    pub const NEUTRAL: Self = Self { pleasure: 0.0, arousal: 0.5, dominance: 0.5 };
    pub const JOY: Self = Self { pleasure: 1.0, arousal: 0.5, dominance: 0.5 };
    pub const FEAR: Self = Self { pleasure: -0.7, arousal: 0.8, dominance: 0.2 };
    pub const ANGER: Self = Self { pleasure: -0.5, arousal: 0.8, dominance: 0.7 };
    pub const SADNESS: Self = Self { pleasure: -0.7, arousal: 0.2, dominance: 0.3 };
}
```

#### 2.2.2 情绪智能系统

```rust
pub struct EmotionalIntelligence {
    current: Pad,                 // 当前情绪状态
    baseline: Pad,                // 基线情绪
    history: Vec<(u64, Pad)>,     // 情绪历史 (时间戳, 状态)
    decay_rate: f32,              // 衰减率
}

impl EmotionalIntelligence {
    /// 处理情绪事件
    pub fn update(&mut self, event: &EmotionalEvent) {
        // 1. 记录历史
        self.history.push((Self::now(), self.current));
        
        // 2. 更新当前状态 (带边界检查)
        self.current.pleasure = (self.current.pleasure + event.pleasure_impact)
            .clamp(-1.0, 1.0);
        self.current.arousal = (self.current.arousal + event.arousal_impact)
            .clamp(0.0, 1.0);
        self.current.dominance = (self.current.dominance + event.dominance_impact)
            .clamp(0.0, 1.0);
    }

    /// 情绪衰减 (随时间回归基线)
    pub fn tick(&mut self) {
        self.current.decay(&self.baseline, self.decay_rate);
    }

    /// 共情 (情绪传染)
    pub fn empathize(&mut self, other: &Pad) {
        let contagion_factor = 0.3;  // 传染系数
        self.current.pleasure = self.current.pleasure * (1.0 - contagion_factor) 
            + other.pleasure * contagion_factor;
        self.current.arousal = self.current.arousal * (1.0 - contagion_factor) 
            + other.arousal * contagion_factor;
    }
}
```

#### 2.2.3 情绪与基本情绪转换

```rust
/// 16种基本情绪 (基于 Russell 情绪环)
pub enum BasicEmotion {
    Excited, Delighted, Happy, Content,
    Surprised, Angry, Afraid, Distressed,
    Sad, Bored, Relaxed, Depressed,
    Disgusted, Anxious, Serene, Confident,
}

impl Pad {
    /// PAD -> 基本情绪 (最近邻匹配)
    pub fn to_basic_emotion(self) -> BasicEmotion {
        let emotions = [/* 所有16种情绪 */];
        
        emotions.iter()
            .min_by(|&&a, &&b| {
                let pad_a = Pad::from_basic_emotion(a);
                let pad_b = Pad::from_basic_emotion(b);
                let dist_a = self.distance(&pad_a);
                let dist_b = self.distance(&pad_b);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .copied()
            .unwrap_or(BasicEmotion::Content)
    }
}
```

---

### 2.3 Memory 模块 (记忆系统)

**文件位置**: `src/memory/`

#### 2.3.1 多模态记忆架构

```
┌─────────────────────────────────────────────────────┐
│                 UnifiedMemory                       │
├──────────────┬──────────────┬─────────────┬─────────┤
│ ShortTerm    │  Episodic    │  Semantic   │Procedural
│  (STM)       │  (事件记忆)   │  (概念记忆)  │ (技能记忆)
├──────────────┼──────────────┼─────────────┼─────────┤
│ 容量: 7±2    │  长期存储     │  长期存储    │ 长期存储│
│ 时间: 秒~分  │  时间+空间    │  概念网络    │ 步骤序列│
│ 触发: 注意力 │  情绪标记     │  关系推理    │ 执行统计│
└──────────────┴──────────────┴─────────────┴─────────┘
           ↓
    ConsolidationEngine (巩固引擎)
           ↓
    睡眠时 STM → LTM 转移
```

#### 2.3.2 短期记忆 (STM)

```rust
pub struct ShortTermMemory {
    capacity: usize,              // 容量限制 (默认 7)
    items: Vec<MemoryChunk>,      // 记忆块
}

pub struct MemoryChunk {
    pub id: String,
    pub content: String,
    pub priority: Priority,       // Critical/High/Medium/Low
    pub emotional_tag: Option<EmotionalTag>,
    pub rehearsal_count: u32,     // 复述次数 (影响巩固)
    pub timestamp: u64,
}

impl ShortTermMemory {
    /// 添加项目 (满时驱逐低优先级)
    pub fn push(&mut self, content: &str) -> Option<MemoryChunk> {
        if self.items.len() >= self.capacity {
            // 找到最低激活度的项目驱逐
            let evicted = self.items.remove(min_idx);
            self.items.push(new_chunk);
            Some(evicted)
        } else {
            self.items.push(new_chunk);
            None
        }
    }

    /// 复述 (增强记忆强度)
    pub fn rehearse(&mut self, id: &str) -> Result<(), MemoryError> {
        if let Some(chunk) = self.items.iter_mut().find(|i| i.id == id) {
            chunk.rehearsal_count += 1;
            chunk.activation = (chunk.activation + 0.1).min(1.0);
            Ok(())
        } else {
            Err(MemoryError::ItemNotFound)
        }
    }

    /// 获取准备巩固的项目
    pub fn ready_for_consolidation(&self, threshold: u32) -> Vec<&MemoryChunk> {
        self.items.iter()
            .filter(|i| i.rehearsal_count >= threshold)
            .collect()
    }
}
```

#### 2.3.3 情节记忆 (Episodic)

```rust
pub struct EpisodicMemory {
    episodes: Vec<Episode>,
}

pub struct Episode {
    pub id: String,
    pub what: String,           // 事件内容
    pub when: u64,              // 时间戳
    pub where_: Option<Location>, // 空间位置
    pub importance: f32,        // 重要性 (0-1)
    pub emotional_valence: Option<EmotionalValence>,
}

impl EpisodicMemory {
    /// 按时间范围查询
    pub fn query_time_range(&self, start: u64, end: u64) -> Vec<&Episode> {
        self.episodes.iter()
            .filter(|e| e.when >= start && e.when <= end)
            .collect()
    }

    /// 按位置查询
    pub fn query_location(&self, location_name: &str) -> Vec<&Episode> {
        self.episodes.iter()
            .filter(|e| e.where_.as_ref()
                .map(|l| l.name == location_name)
                .unwrap_or(false))
            .collect()
    }

    /// 记忆巩固 (摘要生成)
    pub fn consolidate(&mut self, time_range: (u64, u64)) -> Option<Episode> {
        let episodes = self.query_time_range(time_range.0, time_range.1);
        if episodes.len() >= 3 {
            // 生成摘要事件
            let summary = format!("Summary of {} events", episodes.len());
            Some(Episode::new(summary, time_range.1, None))
        } else {
            None
        }
    }
}
```

#### 2.3.4 语义记忆 (Semantic)

```rust
pub struct SemanticMemory {
    concepts: HashMap<String, Concept>,
    relations: Vec<Relation>,
}

pub struct Concept {
    pub id: String,
    pub name: String,
    pub definition: String,
    pub category: String,
    pub attributes: Vec<Attribute>,
}

pub struct Relation {
    pub source: String,         // 概念A ID
    pub target: String,         // 概念B ID
    pub relation_type: RelationType, // IsA, HasPart, Causes, etc.
    pub strength: f32,          // 关系强度
}

impl SemanticMemory {
    /// 关系推理 (传递闭包)
    pub fn infer_related(&self, concept_id: &str, depth: usize) -> Vec<String> {
        let mut related = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        queue.push_back((concept_id.to_string(), 0));
        
        while let Some((current, d)) = queue.pop_front() {
            if d >= depth || visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());
            
            // 查找所有相关概念
            for rel in &self.relations {
                if rel.source == current {
                    related.push(rel.target.clone());
                    queue.push_back((rel.target.clone(), d + 1));
                }
            }
        }
        
        related
    }
}
```

#### 2.3.5 记忆查询系统

```rust
pub struct MemoryQuery {
    pub query: String,                    // 搜索文本
    pub memory_types: Vec<MemoryType>,    // 记忆类型过滤
    pub time_range: Option<(u64, u64)>,   // 时间范围
    pub location: Option<String>,         // 位置过滤
    pub min_importance: f32,              // 重要性阈值
    pub limit: usize,                     // 结果限制
    pub emotional_filter: Option<EmotionalFilter>,
}

pub struct UnifiedMemory {
    pub short_term: ShortTermMemory,
    pub episodic: EpisodicMemory,
    pub semantic: SemanticMemory,
    pub procedural: ProceduralMemory,
    pub consolidation: ConsolidationEngine,
}

impl UnifiedMemory {
    /// 跨记忆类型统一查询
    pub fn query(&self, query: &MemoryQuery) -> BrainResult<MemoryResults> {
        let mut results = MemoryResults::default();

        for memory_type in &query.memory_types {
            match memory_type {
                MemoryType::ShortTerm => {
                    results.short_term = self.short_term.retrieve(&query.query);
                }
                MemoryType::Episodic => {
                    results.episodic = self.episodic.search(&query.query)
                        .into_iter()
                        .filter(|e| e.importance >= query.min_importance)
                        .collect();
                }
                MemoryType::Semantic => {
                    if let Some(concept) = self.semantic.find_by_name(&query.query) {
                        results.semantic.push(concept);
                    }
                }
                MemoryType::Procedural => {
                    results.procedural = self.procedural.search(&query.query);
                }
            }
        }

        Ok(results)
    }
}
```

---

### 2.4 Cognition 模块 (认知系统)

**文件位置**: `src/cognition/`

#### 2.4.1 认知状态架构

```rust
pub struct CognitiveState {
    pub attention_focus: Vec<String>,      // 当前注意力焦点
    pub working_memory: WorkingMemory,     // 工作记忆
    pub goals: Vec<Goal>,                  // 目标栈
    pub current_intention: Option<Intention>, // 当前意图
}

pub struct WorkingMemory {
    capacity: usize,                       // 容量限制 (通常 10)
    items: Vec<MemoryItem>,
    decay_rate: f32,                       // 遗忘速率
}

pub struct Goal {
    pub id: String,
    pub description: String,
    pub priority: f32,                     // 优先级 (0-1)
    pub deadline: Option<u64>,
    pub subgoals: Vec<Goal>,
    pub status: GoalStatus,                // Active/Suspended/Achieved/Failed
}

pub struct Intention {
    pub id: String,
    pub goal_id: String,
    pub plan: Vec<Action>,                 // 行动计划
    pub status: IntentionStatus,
}
```

#### 2.4.2 目标管理逻辑

```rust
impl CognitiveState {
    /// 添加目标 (自动按优先级排序)
    pub fn set_goal(&mut self, goal: Goal) {
        self.goals.push(goal);
        // 降序排序: 高优先级在前
        self.goals.sort_by(|a, b| {
            b.priority.partial_cmp(&a.priority).unwrap()
        });
    }

    /// 形成意图 (从最高优先级目标)
    pub fn form_intention(&mut self) -> Option<Intention> {
        self.goals.first().map(|goal| {
            let intention = Intention {
                id: Uuid::new_v4().to_string(),
                goal_id: goal.id.clone(),
                plan: self.plan_for_goal(goal),
                status: IntentionStatus::Formed,
            };
            self.current_intention = Some(intention.clone());
            intention
        })
    }
}
```

---

### 2.5 Personality 模块 (人格系统)

**文件位置**: `src/personality/`

#### 2.5.1 OCEAN 模型

```rust
/// 大五人格 (Big Five / OCEAN)
pub struct OceanProfile {
    pub openness: f32,          // 开放性 (创造性)
    pub conscientiousness: f32, // 尽责性 (组织性)
    pub extraversion: f32,      // 外向性 (社交性)
    pub agreeableness: f32,     // 宜人性 (合作性)
    pub neuroticism: f32,       // 神经质 (情绪稳定性)
}

impl OceanProfile {
    /// 预设人格类型
    pub fn creative() -> Self { /* 高开放性 */ }
    pub fn analytical() -> Self { /* 高尽责性 */ }
    pub fn leader() -> Self { /* 高外向性+尽责性 */ }
    
    /// 人格影响决策
    pub fn influence(&self, behavior: &mut Behavior) {
        behavior.creativity = self.openness;
        behavior.risk_tolerance = 1.0 - self.neuroticism;
        behavior.social_drive = self.extraversion;
    }
}
```

#### 2.5.2 人格对情绪的影响

```rust
// api.rs 中的实现
fn modify_by_personality(&self, stimulus: Pad, _intensity: f32) -> Pad {
    let o = self.personality.openness;
    let n = self.personality.neuroticism;
    
    // 高神经质放大负面情绪
    let pleasure_mod = if stimulus.pleasure < 0.0 {
        stimulus.pleasure * (1.0 + n * 0.5)
    } else {
        stimulus.pleasure
    };
    
    // 高开放性增加唤醒度
    let arousal_mod = stimulus.arousal * (1.0 + o * 0.2);
    
    Pad::new(pleasure_mod, arousal_mod, stimulus.dominance)
}
```

---

## 数据流转分析

### 3.1 刺激处理流程

```
外部刺激 (文本/事件)
    ↓
┌────────────────────────────────────────────────────────────┐
│ SocialBrainApi::process_stimulus()                         │
│                                                            │
│  1. store_memory(stimulus, importance=0.5)                 │
│     → UnifiedMemory::short_term.push()                     │
│                                                            │
│  2. analyze_sentiment(stimulus)                            │
│     → 关键词匹配 → PAD 情绪值                             │
│                                                            │
│  3. apply_emotional_stimulus(pad, intensity=0.3)           │
│     → modify_by_personality()  [人格调节]                 │
│     → EmotionalIntelligence::update()                      │
│                                                            │
│  4. 神经网络处理 (如果可用)                                │
│     → NeuralNetwork::predict(inputs)                       │
│     → decode_response(outputs)                             │
│                                                            │
│  5. infer_goal(stimulus)                                   │
│     → 关键词匹配 → 创建 Goal                             │
│     → CognitiveState::set_goal()                           │
│                                                            │
│  6. suggest_action()                                       │
│     → 基于当前情绪状态推荐行动                            │
│                                                            │
└────────────────────────────────────────────────────────────┘
    ↓
StimulusResponse {
    memory_id,
    emotional_change,
    response,
    action_recommended,
}
```

### 3.2 进化学习流程

```
环境反馈 (奖励/惩罚)
    ↓
Population::evolve(fitness_results)
    ↓
┌────────────────────────────────────────────────────────────┐
│ 1. 分配适应度到基因组                                      │
│    genome.fitness = fitness_results[genome.id]             │
│                                                            │
│ 2. 物种形成 (Speciation)                                   │
│    for each genome:                                        │
│      distance = genome.compatibility_distance(representative)
│      if distance < threshold:                              │
│         加入该物种                                         │
│      else:                                                 │
│         创建新物种                                         │
│                                                            │
│ 3. 适应度调整 (Fitness Sharing)                            │
│    adjusted_fitness = fitness / species.members.len()     │
│    [防止单一物种垄断]                                      │
│                                                            │
│ 4. 计算后代数量                                            │
│    species.offspring = (species_fitness / total_fitness) * pop_size
│                                                            │
│ 5. 精英保留                                                │
│    每个物种保留最佳个体                                    │
│                                                            │
│ 6. 产生后代                                                │
│    while new_population.size < target:                     │
│       parent1, parent2 = select_parents(species)           │
│       child = Genome::crossover(parent1, parent2)          │
│       child.mutate(config)                                 │
│       new_population.push(child)                           │
│                                                            │
└────────────────────────────────────────────────────────────┘
    ↓
新一代种群
```

### 3.3 记忆巩固流程

```
睡眠/休息触发
    ↓
UnifiedMemory::consolidate()
    ↓
┌────────────────────────────────────────────────────────────┐
│ 1. 获取准备巩固的 STM 项目                                 │
│    ready = short_term.ready_for_consolidation(threshold=3) │
│                                                            │
│ 2. 对每个项目判断类型                                      │
│    for chunk in ready:                                     │
│       if chunk.content.len() > 50:                         │
│          → 情节记忆 (Episodic)                             │
│          episodic.encode(chunk.content, timestamp, location)
│       else if 概念性内容:                                  │
│          → 语义记忆 (Semantic)                             │
│          semantic.learn_concept(chunk.content, ...)        │
│                                                            │
│ 3. 从 STM 移除已巩固项目                                   │
│    short_term.remove_all(ready)                            │
│                                                            │
└────────────────────────────────────────────────────────────┘
    ↓
长期记忆更新
```

---

## 模块间关系

### 4.1 依赖关系图

```
beebotos-social-brain
├── beebotos-core (AgentId)
├── serde (序列化)
├── rand (随机数)
├── tracing (日志)
├── petgraph (图算法)
├── rayon (并行计算)
├── uuid (唯一ID)
└── chrono (时间处理)

内部模块依赖:
┌─────────────────────────────────────────────────────────────┐
│ api.rs (统一入口)                                           │
│  ├── neat/* (神经网络推理)                                  │
│  ├── pad/* (情绪处理)                                       │
│  ├── memory/* (记忆存取)                                    │
│  ├── cognition/* (目标管理)                                 │
│  ├── personality/* (人格影响)                               │
│  └── emotion/* (情绪状态)                                   │
├─────────────────────────────────────────────────────────────┤
│ 数据流向:                                                   │
│                                                             │
│  Input → [api] → [emotion/pad] → [cognition] → [memory]    │
│                ↓                    ↓                        │
│           [personality]        [neat] (决策)                │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 调用关系表

| 调用方 | 被调用方 | 用途 |
|-------|---------|------|
| `api::process_stimulus` | `memory::store_memory` | 存储刺激 |
| `api::process_stimulus` | `pad::analyze_sentiment` | 情绪分析 |
| `api::apply_emotional_stimulus` | `personality::modify_by_personality` | 人格调节 |
| `api::think` | `neat::NeuralNetwork::predict` | 神经网络推理 |
| `api::set_goal` | `cognition::CognitiveState::set_goal` | 目标管理 |
| `api::consolidate_memories` | `memory::UnifiedMemory::consolidate` | 记忆巩固 |
| `neat::Population::evolve` | `neat::Genome::crossover/mutate` | 进化操作 |
| `pad::EmotionalIntelligence::update` | `pad::Pad::clamp` | 边界检查 |
| `memory::consolidate` | `memory::ShortTermMemory::ready_for_consolidation` | 筛选项目 |

---

## 函数接口体系

### 5.1 公共 API (api.rs)

```rust
/// 主要入口
impl SocialBrainApi {
    // 构造方法
    pub fn new() -> Self;
    pub fn with_config(config: ApiConfig) -> Self;
    pub fn with_genome(self, genome: &Genome) -> Self;

    // 记忆接口
    pub fn query_memory(&self, query: &MemoryQuery) -> BrainResult<MemoryResults>;
    pub fn store_memory(&mut self, content: &str, importance: f32) -> BrainResult<String>;
    pub fn consolidate_memories(&mut self) -> BrainResult<usize>;

    // 情绪接口
    pub fn current_emotion(&self) -> EmotionState;
    pub fn current_pad(&self) -> Pad;
    pub fn apply_emotional_stimulus(&mut self, stimulus: Pad, intensity: f32);

    // 认知接口
    pub fn process_stimulus(&mut self, stimulus: &str) -> BrainResult<StimulusResponse>;
    pub fn set_goal(&mut self, description: &str, priority: f32) -> String;
    pub fn current_goals(&self) -> Vec<&Goal>;
    pub fn form_intention(&mut self) -> Option<Intention>;

    // 人格接口
    pub fn personality(&self) -> &OceanProfile;
    pub fn set_personality(&mut self, personality: OceanProfile);

    // 神经网络接口
    pub fn think(&self, inputs: &[f32]) -> Option<Vec<f32>>;

    // 统计接口
    pub fn stats(&self) -> ApiStats;
}
```

### 5.2 NEAT 接口

```rust
// Genome 操作
impl Genome {
    pub fn new_minimal(id: u64, input_size: usize, output_size: usize) -> Self;
    pub fn crossover(parent1: &Genome, parent2: &Genome) -> Genome;
    pub fn mutate(&mut self, config: &NeatConfig, innovations: &mut InnovationTracker);
    pub fn compatibility_distance(&self, other: &Genome, config: &NeatConfig) -> f32;
    pub fn node_count(&self) -> usize;
    pub fn enabled_connections(&self) -> Vec<&ConnectionGene>;
}

// Population 操作
impl Population {
    pub fn new(size: usize, input_size: usize, output_size: usize, config: &NeatConfig) -> Self;
    pub fn speciate(&mut self, config: &NeatConfig);
    pub fn evolve(&mut self, fitness_results: &[FitnessResult], config: &NeatConfig);
    pub fn stats(&self) -> PopulationStats;
}

// NeuralNetwork 操作
impl NeuralNetwork {
    pub fn from_genome(genome: &Genome) -> Self;
    pub fn forward(&mut self, inputs: &[f32]) -> Vec<f32>;
    pub fn predict(&self, inputs: &[f32]) -> Vec<f32>;
    pub fn activate(&mut self, inputs: &[f32]) -> Vec<f32>;
}
```

### 5.3 Memory 接口

```rust
// ShortTermMemory
impl ShortTermMemory {
    pub fn new() -> Self;
    pub fn with_capacity(capacity: usize) -> Self;
    pub fn push(&mut self, content: &str) -> Option<MemoryChunk>;
    pub fn push_with_priority(&mut self, content: &str, priority: Priority) -> Option<MemoryChunk>;
    pub fn rehearse(&mut self, id: &str) -> Result<(), MemoryError>;
    pub fn retrieve(&self, query: &str) -> Vec<&MemoryChunk>;
    pub fn ready_for_consolidation(&self, threshold: u32) -> Vec<&MemoryChunk>;
}

// EpisodicMemory
impl EpisodicMemory {
    pub fn new() -> Self;
    pub fn encode(&mut self, what: impl Into<String>, when: u64, where_: Option<Location>) -> String;
    pub fn query_time_range(&self, start: u64, end: u64) -> Vec<&Episode>;
    pub fn query_location(&self, location_name: &str) -> Vec<&Episode>;
    pub fn search(&self, keyword: &str) -> Vec<&Episode>;
}

// SemanticMemory
impl SemanticMemory {
    pub fn new() -> Self;
    pub fn learn_concept(&mut self, name: &str, definition: &str, category: &str) -> String;
    pub fn add_relation(&mut self, source: &str, target: &str, relation_type: RelationType, strength: f32) -> Result<(), MemoryError>;
    pub fn find_by_name(&self, name: &str) -> Option<&Concept>;
    pub fn infer_related(&self, concept_id: &str, depth: usize) -> Vec<String>;
}

// UnifiedMemory
impl UnifiedMemory {
    pub fn new() -> Self;
    pub fn query(&self, query: &MemoryQuery) -> BrainResult<MemoryResults>;
    pub fn consolidate(&mut self) -> BrainResult<usize>;
}
```

### 5.4 PAD 接口

```rust
// Pad 基础操作
impl Pad {
    pub const NEUTRAL: Self;
    pub const JOY: Self;
    pub const FEAR: Self;
    pub const ANGER: Self;
    
    pub fn new(pleasure: f32, arousal: f32, dominance: f32) -> Self;
    pub fn neutral() -> Self;
    pub fn intensity(&self) -> f32;
    pub fn distance(&self, other: &Pad) -> f32;
    pub fn blend(&self, other: &Self, factor: f32) -> Self;
    pub fn decay(&mut self, baseline: &Pad, rate: f32);
    pub fn from_basic_emotion(emotion: BasicEmotion) -> Self;
    pub fn to_basic_emotion(self) -> BasicEmotion;
}

// EmotionalIntelligence
impl EmotionalIntelligence {
    pub fn new() -> Self;
    pub fn current(&self) -> &Pad;
    pub fn update(&mut self, event: &EmotionalEvent);
    pub fn tick(&mut self);  // 衰减
    pub fn empathize(&mut self, other: &Pad);
}
```

---

## 关键算法实现

### 6.1 NEAT 交叉算法

```rust
/// 两个父代基因组合并产生子代
pub fn crossover(parent1: &Genome, parent2: &Genome) -> Genome {
    // 假设 parent1 更适应
    let mut child = parent1.clone();
    
    for child_conn in &mut child.connections {
        // 寻找匹配基因 (相同创新号)
        if let Some(p2_conn) = parent2.connections.iter()
            .find(|c| c.innovation_number == child_conn.innovation_number) {
            
            // 匹配基因: 随机选择一方
            if rand::random::<bool>() {
                child_conn.weight = p2_conn.weight;
            }
            // 如果一方禁用，有 75% 概率子代禁用 (保护机制)
            if !child_conn.enabled || !p2_conn.enabled {
                child_conn.enabled = rand::random::<f32>() > 0.25;
            }
        }
        // 不匹配基因: 从更适应的父代继承 (已包含在 clone 中)
    }
    
    child.fitness = 0.0;  // 子代需重新评估
    child.adjusted_fitness = 0.0;
    child.id = rand::random::<u64>();
    child
}
```

### 6.2 情绪混合算法

```rust
/// 两个情绪状态的加权混合
pub fn blend(&self, other: &Self, factor: f32) -> Self {
    // factor: 0.0 = 完全保留 self, 1.0 = 完全采用 other
    Self::new(
        self.pleasure * (1.0 - factor) + other.pleasure * factor,
        self.arousal * (1.0 - factor) + other.arousal * factor,
        self.dominance * (1.0 - factor) + other.dominance * factor,
    )
}

/// 情绪衰减 (向基线回归)
pub fn decay(&mut self, baseline: &Pad, rate: f32) {
    self.pleasure = self.pleasure * (1.0 - rate) + baseline.pleasure * rate;
    self.arousal = self.arousal * (1.0 - rate) + baseline.arousal * rate;
    self.dominance = self.dominance * (1.0 - rate) + baseline.dominance * rate;
}
```

### 6.3 记忆优先级排序

```rust
/// 基于多因素的优先级计算
fn calculate_priority(&self, chunk: &MemoryChunk) -> f32 {
    let time_decay = (now() - chunk.timestamp) as f32 / 3600.0; // 小时
    let rehearsal_boost = (chunk.rehearsal_count as f32).ln_1p();
    let emotional_boost = chunk.emotional_tag.as_ref()
        .map(|e| e.intensity * e.salience)
        .unwrap_or(0.0);
    
    // 优先级 = 基础优先级 × 时间衰减 + 复述加成 + 情绪加成
    let base = chunk.priority.value(); // Critical=1.0, High=0.75, Medium=0.5, Low=0.25
    base * (-0.1 * time_decay).exp() + 0.1 * rehearsal_boost + 0.2 * emotional_boost
}
```

### 6.4 人格影响计算

```rust
/// 人格特质调节情绪响应
fn apply_personality_influence(
    stimulus: Pad, 
    personality: &OceanProfile
) -> Pad {
    // 神经质: 放大负面情绪的强度和持续时间
    let pleasure_mod = if stimulus.pleasure < 0.0 {
        stimulus.pleasure * (1.0 + personality.neuroticism * 0.5)
    } else {
        stimulus.pleasure * (1.0 - personality.neuroticism * 0.1)
    };
    
    // 开放性: 增加对新奇刺激的情绪反应
    let arousal_mod = stimulus.arousal * (1.0 + personality.openness * 0.2);
    
    // 外向性: 增强正性情绪
    let pleasure_mod = if stimulus.pleasure > 0.0 {
        pleasure_mod * (1.0 + personality.extraversion * 0.15)
    } else {
        pleasure_mod
    };
    
    // 宜人性: 减少愤怒/敌意情绪
    let pleasure_mod = if stimulus.pleasure < -0.3 {
        pleasure_mod * (1.0 - personality.agreeableness * 0.2)
    } else {
        pleasure_mod
    };
    
    Pad::new(pleasure_mod, arousal_mod, stimulus.dominance)
}
```

---

## 状态管理机制

### 7.1 配置分层

```rust
/// 全局配置分层体系
pub struct BrainConfig {
    pub neat: NeatConfig,       // NEAT 专用配置
    pub pad: PadConfig,         // PAD 配置
    pub memory: MemoryConfig,   // 记忆配置
    pub personality: PersonalityConfig, // 人格配置
    pub parallel: ParallelConfig, // 并行配置
    pub features: FeatureToggles, // 功能开关
}

impl BrainConfig {
    /// 轻量级配置 (低资源消耗)
    pub fn lightweight() -> Self {
        Self {
            neat: NeatConfig::conservative(),
            memory: MemoryConfig {
                stm_capacity: 5,
                consolidation_threshold: 5,
                ..Default::default()
            },
            parallel: ParallelConfig { enabled: false, .. },
            features: FeatureToggles {
                learning: false,
                social: false,
                metacognition: false,
                creativity: false,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// 高性能配置
    pub fn high_performance() -> Self {
        Self {
            neat: NeatConfig::aggressive(),
            memory: MemoryConfig {
                stm_capacity: 9,
                consolidation_threshold: 2,
                ..Default::default()
            },
            parallel: ParallelConfig {
                enabled: true,
                worker_threads: 4,
                min_batch_size: 50,
            },
            features: FeatureToggles {
                detailed_logging: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
```

### 7.2 错误处理策略

```rust
/// 分层错误类型
pub enum BrainError {
    MemoryError(String),      // 记忆操作错误
    InvalidState(String),     // 无效状态
    EvolutionError(String),   // 进化错误
    EmotionError(String),     // 情绪处理错误
    ReasoningError(String),   // 推理错误
    NotFound(String),         // 项目未找到
    InvalidParameter(String), // 无效参数
    ConfigError(String),      // 配置错误
    NotImplemented(String),   // 未实现功能
}

/// 使用 Result 类型别名
pub type BrainResult<T> = Result<T, BrainError>;

/// 错误处理示例
impl SocialBrainApi {
    pub fn query_memory(&self, query: &MemoryQuery) -> BrainResult<MemoryResults> {
        if !self.config.memory_enabled {
            return Ok(MemoryResults::default());
        }
        
        tracing::debug!("Querying memory: {:?}", query);
        
        self.memory.query(query)
            .map_err(|e| BrainError::MemoryError(e.to_string()))
    }
}
```

---

## 扩展与集成

### 8.1 与上层模块集成

```rust
// 被 agents crate 使用
use beebotos_social_brain::{
    SocialBrainApi, BrainConfig,
    neat::{Population, NeatConfig},
};

pub struct CognitiveAgent {
    brain: SocialBrainApi,
    genome_id: Option<u64>,
}

impl CognitiveAgent {
    pub fn new(config: BrainConfig) -> Self {
        Self {
            brain: SocialBrainApi::with_config(config.into()),
            genome_id: None,
        }
    }

    pub fn perceive(&mut self, stimulus: &str) -> AgentResponse {
        let response = self.brain.process_stimulus(stimulus).unwrap();
        
        AgentResponse {
            action: response.action_recommended,
            emotion: response.emotional_change,
            memory_id: response.memory_id,
        }
    }
    
    pub fn evolve(&mut self, fitness: f32, population: &mut Population) {
        // 将适应度反馈给 NEAT 种群
        // ...
    }
}
```

### 8.2 与下层模块关系

```rust
// 依赖 beebotos-core 的基础类型
use beebotos_core::AgentId;

// 在 FitnessResult 中使用
pub struct FitnessResult {
    pub agent_id: AgentId,  // 来自 core 层
    pub fitness: f32,
    pub generation: usize,
    pub metrics: HashMap<String, f32>,
}
```

### 8.3 扩展点

| 扩展类型 | 方式 | 示例 |
|---------|------|------|
| 新记忆类型 | 实现 Memory trait | TemporalMemory |
| 新情绪模型 | 替换 PAD 实现 | OCCModel |
| 新学习算法 | 添加到 learning 模块 | ActorCritic |
| 新推理类型 | 添加到 reasoning 模块 | AnalogicalReasoning |
| 新配置模式 | 扩展 BrainConfig | RealTimeConfig |

---

## 附录

### A. 术语表

| 术语 | 英文 | 说明 |
|-----|------|------|
| 基因型 | Genotype | 基因编码表示 (Genome) |
| 表现型 | Phenotype | 实际表达的神经网络 |
| 创新号 | Innovation Number | NEAT 中追踪基因历史的全局标记 |
| 物种形成 | Speciation | 基于兼容性距离将种群分组 |
| 适应度共享 | Fitness Sharing | 同物种内竞争适应度的机制 |
| PAD | Pleasure-Arousal-Dominance | 三维情绪模型 |
| 复述 | Rehearsal | 强化短期记忆的过程 |
| 巩固 | Consolidation | 短期记忆转长期记忆的过程 |
| OCEAN | Openness-Conscientiousness-Extraversion-Agreeableness-Neuroticism | 大五人格模型 |

### B. 性能特征

| 操作 | 时间复杂度 | 空间复杂度 | 备注 |
|-----|-----------|-----------|------|
| 网络前向传播 | O(n) | O(1) | n = 连接数 |
| 基因组交叉 | O(c) | O(c) | c = 基因数 |
| 兼容性距离 | O(c1 + c2) | O(1) | 两基因组基因数 |
| 记忆查询 | O(m) | O(r) | m = 记忆数, r = 结果数 |
| 情绪更新 | O(1) | O(1) | 固定操作 |
| 物种形成 | O(p × s) | O(s) | p = 种群大小, s = 物种数 |

---

**文档版本**: v1.0  
**最后更新**: 2026-03-23  
**架构版本**: BeeBotOS-V2  
**维护者**: BeeBotOS Team

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
我已为 beebotos-social-brain 模块创建了详细的架构和代码逻辑分析文档。

---

## 📄 文档已生成

**文件位置**: `docs/social-brain-architecture-logic.md`

**文档规模**: 45,874 字节，约 1,200 行

---

## 📋 文档内容概览

### 1. 架构概述
```
┌─────────────────────────────────────────────────────────────┐
│  🧠 Social Brain Layer (beebotos-social-brain)             │
│  ┌──────────┬──────────┬──────────┬──────────┐             │
│  │  NEAT    │   PAD    │  Memory  │Cognition │             │
│  │(神经进化)│ (情绪)   │ (记忆)   │ (认知)   │             │
│  └──────────┴──────────┴──────────┴──────────┘             │
└─────────────────────────────────────────────────────────────┘
```

### 2. 核心模块详解

| 模块 | 核心结构 | 关键算法 |
|------|---------|---------|
| **NEAT** | Genome / NeuralNetwork / Population | 兼容性距离、交叉对齐、创新号追踪 |
| **PAD** | Pad / EmotionalIntelligence | 三维空间情绪、衰减模型、共情传染 |
| **Memory** | STM / Episodic / Semantic / Procedural | 优先级驱逐、时间/空间查询、关系推理 |
| **Cognition** | CognitiveState / WorkingMemory / Goal | 优先级排序、意图形成、计划生成 |

### 3. 数据流转分析

包含详细的数据流图：
- **刺激处理流程**: `process_stimulus()` 的 6 步处理逻辑
- **进化学习流程**: 物种形成 → 适应度调整 → 选择 → 交叉 → 变异
- **记忆巩固流程**: STM → LTM 的转移判断逻辑

### 4. 模块间关系

```
内部模块依赖:
┌─────────────────────────────────────────────────────────────┐
│ api.rs (统一入口)                                           │
│  ├── neat/* (神经网络推理)                                  │
│  ├── pad/* (情绪处理)                                       │
│  ├── memory/* (记忆存取)                                    │
│  ├── cognition/* (目标管理)                                 │
│  ├── personality/* (人格影响)                               │
│  └── emotion/* (情绪状态)                                   │
└─────────────────────────────────────────────────────────────┘
```

### 5. 函数接口体系

完整接口定义：
- **SocialBrainApi**: 15+ 公共方法
- **NEAT**: Genome / Population / NeuralNetwork 操作
- **Memory**: STM / Episodic / Semantic / Unified 查询
- **PAD**: 基础操作 / EmotionalIntelligence 更新

### 6. 关键算法实现

包含伪代码和逻辑说明：
- **NEAT 交叉算法**: 基于创新号对齐的基因组合
- **情绪混合算法**: 加权混合 + 衰减回归
- **记忆优先级**: 多因素计算公式
- **人格影响**: 五因素调节模型

### 7. 状态管理机制

- **配置分层**: 轻量级 / 标准 / 高性能三种预设
- **错误处理**: BrainError 枚举 + BrainResult 类型别名

### 8. 扩展与集成

- 与上层 agents 模块的集成示例
- 与下层 core 模块的类型依赖
- 扩展点说明表

---

## 🔑 核心架构图

```
外部刺激 (文本/事件)
    ↓
SocialBrainApi::process_stimulus()
    ├── 1. store_memory() → UnifiedMemory::short_term
    ├── 2. analyze_sentiment() → PAD 情绪值
    ├── 3. apply_emotional_stimulus() → 人格调节 → 情绪更新
    ├── 4. NeuralNetwork::predict() → 神经网络推理 (可选)
    ├── 5. infer_goal() → CognitiveState::set_goal()
    └── 6. suggest_action() → 基于情绪推荐行动
    ↓
StimulusResponse { memory_id, emotional_change, response, action }
```

---

## 📊 关键数据关系

| 概念 | 关系 | 说明 |
|-----|------|------|
| Genome ↔ NeuralNetwork | 基因型 → 表现型 | `NeuralNetwork::from_genome()` 转换 |
| STM ↔ LTM | 短期 → 长期 | `rehearsal_count >= threshold` 触发巩固 |
| PAD ↔ BasicEmotion | 连续 ↔ 离散 | 欧氏距离最近邻匹配 |
| Personality ↔ Emotion | 调节关系 | 神经质放大负面、开放性增加唤醒 |
| Goal ↔ Intention | 目标 → 意图 | `form_intention()` 形成具体计划 |

这份文档涵盖了模块的完整工作原理、代码逻辑、业务关系和接口定义，可直接用于架构设计和二次开发参考。

