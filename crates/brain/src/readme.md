## beebotos-social-brain 编译和使用指南

**beebotos-social-brain** 是 BeeBotOS 的 **社交大脑层**，提供认知架构功能，包括 NEAT 神经网络进化、PAD 情绪模型、OCEAN 人格模型、多模态记忆系统和演绎推理能力。

---

### 📦 编译命令

#### 1. 编译整个项目（包含 social-brain）
```bash
# 项目根目录
cargo build --release

# 编译后的库
# target/release/libbeebotos_social_brain.rlib
```

#### 2. 只编译 Social Brain crate
```bash
# 编译 beebotos-social-brain
cargo build --release -p beebotos-social-brain

# 调试模式
cargo build -p beebotos-social-brain
```

#### 3. 运行基准测试
```bash
# 运行性能基准
cargo bench -p beebotos-social-brain

# 生成 HTML 报告
# 位于 target/criterion/pad_bench/report/index.html
```

#### 4. 运行测试
```bash
# 运行单元测试
cargo test -p beebotos-social-brain

# 带日志输出
RUST_LOG=debug cargo test -p beebotos-social-brain -- --nocapture
```

---

### 🚀 使用方法

#### 作为库依赖

在 `Cargo.toml` 中添加：
```toml
[dependencies]
beebotos-social-brain = { path = "crates/social-brain" }
```

---

### 💻 编程示例

#### 1. NEAT 神经网络进化

```rust
use beebotos_social_brain::neat::{
    Population, NeatConfig, Genome, NeuralNetwork, 
    InnovationTracker, FitnessResult
};
use beebotos_core::AgentId;

fn main() {
    // 配置 NEAT 参数
    let config = NeatConfig::standard();
    
    // 创建初始种群
    let mut population = Population::new(
        150,      // 种群大小
        3,        // 输入节点数
        2,        // 输出节点数
        &config,
    );
    
    // 进化多代
    for generation in 0..100 {
        // 评估适应度（模拟）
        let fitness_results: Vec<FitnessResult> = population.genomes
            .iter()
            .map(|g| {
                // 创建神经网络并评估
                let network = NeuralNetwork::from_genome(g);
                let fitness = evaluate_network(&network);
                
                FitnessResult {
                    agent_id: AgentId::new(&format!("agent_{}", g.id)),
                    fitness,
                    generation,
                    metrics: Default::default(),
                }
            })
            .collect();
        
        // 执行进化
        population.evolve(&fitness_results, &config);
        
        // 打印统计
        let stats = population.stats();
        println!(
            "Gen {}: Best={:.2}, Avg={:.2}, Species={}",
            stats.generation,
            stats.best_fitness,
            stats.avg_fitness,
            stats.species_count
        );
    }
    
    // 获取最佳基因组
    if let Some(best) = &population.best_genome {
        println!("Best genome ID: {}", best.id);
    }
}

fn evaluate_network(network: &NeuralNetwork) -> f32 {
    // 测试输入
    let inputs = vec![1.0, 0.5, -0.5];
    let outputs = network.activate(&inputs);
    
    // 根据任务计算适应度
    // 这里使用示例逻辑
    let target = vec![0.8, 0.2];
    let error: f32 = outputs.iter()
        .zip(target.iter())
        .map(|(o, t)| (o - t).powi(2))
        .sum();
    
    1.0 / (1.0 + error) // 转换误差为适应度
}
```

---

#### 2. PAD 情绪模型

```rust
use beebotos_social_brain::pad::{
    Pad, Emotion, EmotionalIntelligence, 
    EmotionalTrait, EmotionCategory
};

fn main() {
    // 创建情绪状态
    let mut emotion = Emotion::new(
        Pad::new(0.5, 0.3, 0.2)  // 愉悦度, 唤醒度, 支配度
    );
    
    println!("当前情绪: {}", emotion.current_label());
    
    // 更新情绪（受刺激影响）
    emotion.update(Pad::new(0.2, -0.1, 0.0));
    
    // 获取基本情绪类型
    if let Some(basic) = emotion.basic_emotion() {
        println!("基本情绪: {:?}", basic);
    }
    
    // 情绪特质影响（长期性格）
    let trait_optimistic = EmotionalTrait::Optimistic;
    let baseline = trait_optimistic.baseline_offset();
    println!("乐观特质基线偏移: {:?}", baseline);
    
    // 情绪轮分类
    let category = EmotionCategory::Joy;
    let center = category.pad_center();
    println!("喜悦在PAD空间中心: {:?}", center);
    
    // 创建情绪智能
    let mut ei = EmotionalIntelligence::new();
    
    // 记录情绪事件
    ei.record_event(
        "task_completed",
        Pad::new(0.6, 0.2, 0.3),
        0.8, // 强度
    );
    
    // 检测情绪趋势
    let trend = ei.detect_trend(10); // 最近10个事件
    println!("情绪趋势: {:?}", trend);
}
```

---

#### 3. OCEAN 人格模型

```rust
use beebotos_social_brain::personality::{
    OceanProfile, OceanEngine, Experience, Outcome
};

fn main() {
    // 创建人格档案
    let profile = OceanProfile::new(
        0.7,  // 开放性 (Openness)
        0.6,  // 尽责性 (Conscientiousness)
        0.5,  // 外向性 (Extraversion)
        0.8,  // 宜人性 (Agreeableness)
        0.3,  // 神经质 (Neuroticism)
    );
    
    // 或使用预设
    let creative = OceanProfile::creative();
    let analytical = OceanProfile::analytical();
    let leader = OceanProfile::leader();
    
    // 计算人格相似度
    let similarity = profile.similarity(&creative);
    println!("与创意型人格相似度: {:.2}", similarity);
    
    // 主导特质
    let (trait_name, value) = profile.dominant_trait();
    println!("主导特质: {} ({:.2})", trait_name, value);
    
    // 影响行为
    let mut behavior = Behavior::default();
    profile.influence(&mut behavior);
    println!("创造力: {:.2}", behavior.creativity);
    println!("风险承受: {:.2}", behavior.risk_tolerance);
    
    // 人格引擎（动态适应）
    let mut engine = OceanEngine::new(profile);
    
    // 根据经验适应
    let experience = Experience {
        outcome: Outcome::Positive,
        openness_relevance: 0.8,
        context: "成功解决复杂问题".to_string(),
    };
    engine.adapt(&experience);
    
    // 决策风格
    let style = engine.decision_style();
    println!("决策风格: {:?}", style);
    
    // 学习策略
    let strategy = engine.learning_strategy();
    println!("学习策略: {:?}", strategy);
}
```

---

#### 4. 记忆系统

```rust
use beebotos_social_brain::memory::{
    ShortTermMemory, EpisodicMemory, SemanticMemory,
    MemoryChunk, Priority, Episode, Concept
};

fn main() {
    // 短期记忆（7±2 项目）
    let mut stm = ShortTermMemory::new(7);
    
    // 添加记忆块
    stm.add(MemoryChunk {
        content: "重要电话号码".to_string(),
        priority: Priority::High,
        emotional_tag: Some(EmotionalTag::Important),
    });
    
    // 检查记忆负荷
    println!("负载: {}/{}", stm.load(), stm.capacity());
    
    // 情节记忆（带时空上下文的事件）
    let mut episodic = EpisodicMemory::new();
    
    episodic.record(Episode {
        timestamp: chrono::Utc::now(),
        location: Location::new("会议室A", (40.7, -74.0)),
        event: "项目启动会议".to_string(),
        participants: vec!["Alice".to_string(), "Bob".to_string()],
        outcome: Outcome::Positive,
    });
    
    // 按时间和位置检索
    let episodes = episodic.retrieve_by_time(
        chrono::Utc::now() - chrono::Duration::days(7),
        chrono::Utc::now(),
    );
    
    // 语义记忆（概念和事实）
    let mut semantic = SemanticMemory::new();
    
    semantic.add_concept(Concept {
        name: "区块链".to_string(),
        definition: "分布式账本技术".to_string(),
        category: "技术".to_string(),
        attributes: vec!["去中心化".to_string(), "不可篡改".to_string()],
    });
    
    // 建立关系
    semantic.add_relation(
        "区块链",
        "包含",
        "智能合约",
        RelationType::HasPart,
    );
    
    // 推理查询
    let related = semantic.infer_related("区块链", 2); // 2跳关系
}
```

---

#### 5. 演绎推理

```rust
use beebotos_social_brain::reasoning::deductive::{
    KnowledgeBase, Rule, Statement, DeductiveEngine
};

fn main() {
    // 创建知识库
    let mut kb = KnowledgeBase::new();
    
    // 添加事实
    kb.add_fact(Statement::atom("所有人都是会死的")));
    kb.add_fact(Statement::atom("苏格拉底是人")));
    
    // 添加规则
    let rule = Rule::new(
        vec![
            Statement::atom("X是人"),
            Statement::atom("所有人都是会死的"),
        ],
        Statement::atom("X是会死的"),
    );
    kb.add_rule(rule);
    
    // 推理引擎
    let engine = DeductiveEngine::new(kb);
    
    // 前向推理
    let conclusions = engine.forward_chain();
    for conclusion in conclusions {
        println!("推导: {:?}", conclusion);
    }
    
    // 后向推理（目标导向）
    let goal = Statement::atom("苏格拉底是会死的");
    let proof = engine.backward_chain(&goal);
    
    if let Some(steps) = proof {
        println!("找到证明路径:");
        for step in steps {
            println!("  - {:?}", step);
        }
    }
    
    // 复杂逻辑
    let complex_fact = Statement::atom("下雨")
        .implies(Statement::atom("地面湿"));
    kb.add_fact(complex_fact);
    
    // 全称量词
    let universal = Statement::ForAll(
        "x".to_string(),
        Box::new(Statement::atom("x是偶数").implies(
            Statement::atom("x能被2整除")
        ))
    );
    kb.add_fact(universal);
}
```

---

#### 6. 注意力机制

```rust
use beebotos_social_brain::attention::{
    Attention, Focus, FocusType, SelectiveAttention
};

fn main() {
    // 创建注意力系统
    let mut attention = Attention::new();
    
    // 设置焦点
    attention.set_focus(Focus {
        target: "当前任务".to_string(),
        focus_type: FocusType::GoalDirected,
        intensity: 0.8,
    });
    
    // 处理多个刺激
    let stimuli = vec![
        ("邮件通知", 0.3),
        ("紧急警告", 0.9),
        ("背景噪音", 0.1),
    ];
    
    for (stimulus, saliency) in stimuli {
        let attended = attention.process(stimulus, saliency);
        if attended {
            println!("注意到: {}", stimulus);
        }
    }
    
    // 选择性注意（过滤干扰）
    let mut selective = SelectiveAttention::new();
    selective.set_filter(|s| s.contains("重要"));
    
    let items = vec!["重要会议", "广告邮件", "重要更新", "社交通知"];
    for item in items {
        if selective.attend(item) {
            println!("过滤后关注: {}", item);
        }
    }
}
```

---

### 📋 核心功能模块

| 模块 | 路径 | 功能 |
|------|------|------|
| **neat** | `src/neat/` | NEAT 神经网络进化算法 |
| **pad** | `src/pad/` | PAD 三维情绪模型 |
| **personality** | `src/personality/` | OCEAN 大五人格模型 |
| **memory** | `src/memory/` | 多模态记忆系统 |
| **reasoning** | `src/reasoning/` | 演绎/归纳/溯因推理 |
| **attention** | `src/attention/` | 注意力机制 |
| **learning** | `src/learning/` | 强化学习和元学习 |
| **emotion** | `src/emotion/` | 情绪计算和传染 |
| **social** | `src/social/` | 社会认知和心智理论 |
| **cognition** | `src/cognition/` | 认知架构 |
| **knowledge** | `src/knowledge/` | 知识图谱和本体 |

---

### ⚙️ 配置选项

```rust
use beebotos_social_brain::BrainConfig;

let config = BrainConfig {
    neat: NeatConfig {
        population_size: 150,
        mutation_rate: 0.8,
        crossover_rate: 0.75,
        compatibility_threshold: 3.0,
        ..NeatConfig::standard()
    },
    pad_enabled: true,
    memory_enabled: true,
};
```

---

### 📁 项目结构

```
crates/social-brain/
├── Cargo.toml
├── benches/
│   └── pad_bench.rs      # 情绪计算性能测试
└── src/
    ├── lib.rs            # 库入口
    ├── api.rs            # 公共 API
    ├── error.rs          # 错误定义
    ├── neat/             # NEAT 进化算法
    │   ├── genome.rs     # 基因组
    │   ├── network.rs    # 神经网络
    │   ├── species.rs    # 物种分类
    │   └── config.rs     # 配置
    ├── pad/              # PAD 情绪模型
    │   ├── emotion.rs    # 情绪状态
    │   └── transition.rs # 情绪转换
    ├── personality/      # 人格系统
    │   ├── ocean.rs      # OCEAN 模型
    │   └── engine.rs     # 人格引擎
    ├── memory/           # 记忆系统
    │   ├── short_term.rs # 短期记忆
    │   ├── episodic.rs   # 情节记忆
    │   ├── semantic.rs   # 语义记忆
    │   └── consolidation.rs # 记忆巩固
    ├── reasoning/        # 推理系统
    │   ├── deductive.rs  # 演绎推理
    │   ├── inductive.rs  # 归纳推理
    │   └── abductive.rs  # 溯因推理
    ├── attention/        # 注意力
    ├── learning/         # 学习算法
    ├── emotion/          # 情绪计算
    ├── social/           # 社会认知
    ├── cognition/        # 认知架构
    └── knowledge/        # 知识表示
```

---

### 🛠 技术栈

| 组件 | 用途 |
|------|------|
| **petgraph** | 图算法（知识图谱）|
| **rayon** | 并行计算 |
| **serde** | 序列化 |
| **rand** | 随机数生成 |
| **uuid** | 唯一标识 |

---

### ⚠️ 注意事项

1. **NEAT 进化** - 大种群可能需要较长时间收敛
2. **记忆系统** - 需要定期执行记忆巩固（consolidation）
3. **情绪传染** - 多 Agent 交互时注意情绪传播效应
4. **计算成本** - 复杂推理和大型知识图谱可能较重

需要我帮你实现具体的认知功能或提供其他使用示例吗？



