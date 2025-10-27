# 智能默认配置设计

**版本**: v1.8.4+
**日期**: 2025-10-27
**目标**: 零配置启动，智能适配

---

## 🎯 设计目标

### 核心理念
> "最好的配置就是不需要配置" - 极简主义

**目标**:
- ✅ 新用户：零配置启动（只需 API key）
- ✅ 普通用户：智能默认，按需调整
- ✅ 高级用户：完全可控，深度定制

---

## 📊 当前问题分析

### 配置复杂度统计

| 配置节 | 配置项数 | 必须项 | 可选项 | 智能默认 |
|--------|---------|--------|--------|----------|
| llm | 5 | 1 (api_key) | 4 | ✅ |
| features | 6 | 0 | 6 | ✅ |
| memory | 3 | 0 | 3 | ✅ |
| display | 2 | 0 | 2 | ✅ |
| conversation | 6 | 0 | 6 | ✅ |
| likan | 6 | 0 | 6 | ✅ |
| voice | 5 | 0 | 5 | ✅ |
| **总计** | **33** | **1** | **32** | **97%** |

**结论**: 97% 配置项可智能默认！

---

## 🏗️ 智能默认方案

### Level 1: 最小配置（新用户）

**realconsole.minimal.yaml**:
```yaml
llm:
  api_key: ${DEEPSEEK_API_KEY}
```

✅ **仅此一行**！其他全部智能默认

**自动默认值**:
```yaml
# 自动填充
llm:
  provider: deepseek          # 智能检测 API key 类型
  model: deepseek-chat        # 默认模型
  endpoint: https://api.deepseek.com/v1  # 默认端点

features:
  shell_enabled: true         # 默认启用
  tool_calling_enabled: true  # 默认启用

likan:
  enabled: true               # 默认启用
  cycle_interval_secs: 300    # 5分钟
  notification_mode: minimal  # 最小通知

# ... 其他全部默认
```

---

### Level 2: 标准配置（普通用户）

**realconsole.standard.yaml**:
```yaml
llm:
  api_key: ${DEEPSEEK_API_KEY}

# 可选：启用提示符显示
likan:
  show_in_prompt: true

# 可选：启用语音播报
voice:
  enabled: true
```

✅ 只配置真正需要的功能

---

### Level 3: 完整配置（高级用户）

**realconsole.yaml** (当前完整版):
```yaml
# 全部配置项，深度定制
llm: ...
features: ...
likan: ...
# ... 所有配置
```

✅ 完全可控，专家模式

---

## 🔧 实现方案

### 1. 配置层级加载

```rust
pub struct ConfigLoader {
    /// 配置加载优先级
    /// 1. 命令行参数（最高）
    /// 2. 用户配置文件
    /// 3. 智能默认值（最低）
}

impl ConfigLoader {
    pub fn load() -> Result<Config> {
        // 1. 加载智能默认
        let mut config = Config::smart_defaults();

        // 2. 尝试加载用户配置
        if let Some(user_config) = Self::load_user_config()? {
            config.merge(user_config);
        }

        // 3. 应用命令行参数
        config.apply_cli_args();

        Ok(config)
    }
}
```

### 2. 智能检测

```rust
impl Config {
    pub fn smart_defaults() -> Self {
        Self {
            // LLM 智能检测
            llm: LlmConfig::detect_from_env(),

            // 功能智能启用
            features: FeaturesConfig::auto_enable(),

            // 离坎智能配置
            likan: LiKanConfig::adaptive(),

            // ... 其他智能默认
        }
    }
}

impl LlmConfig {
    fn detect_from_env() -> Self {
        // 检测环境变量
        if let Ok(key) = env::var("DEEPSEEK_API_KEY") {
            return Self::deepseek(key);
        }
        if let Ok(key) = env::var("OPENAI_API_KEY") {
            return Self::openai(key);
        }
        // ... 其他检测
        Self::default()
    }
}

impl LiKanConfig {
    fn adaptive() -> Self {
        // 自适应配置
        let usage_level = detect_usage_level();

        Self {
            enabled: true,
            cycle_interval_secs: match usage_level {
                UsageLevel::Heavy => 180,   // 3分钟
                UsageLevel::Normal => 300,  // 5分钟
                UsageLevel::Light => 600,   // 10分钟
            },
            notification_mode: NotificationMode::Minimal,
            show_in_prompt: false,  // 默认不干扰
            // ... 自适应参数
        }
    }
}
```

### 3. 配置验证与修复

```rust
impl Config {
    /// 验证配置并自动修复
    pub fn validate_and_fix(&mut self) -> Vec<String> {
        let mut warnings = Vec::new();

        // 检查必要配置
        if self.llm.api_key.is_empty() {
            warnings.push("⚠️ 未配置 LLM API key，部分功能将受限".to_string());
        }

        // 检查配置合理性
        if self.likan.cycle_interval_secs < 60 {
            warnings.push("⚠️ 炼化炉间隔过短，自动调整为 60 秒".to_string());
            self.likan.cycle_interval_secs = 60;
        }

        // 自动修复冲突
        if self.likan.notification_mode == NotificationMode::Prompt
            && !self.likan.show_in_prompt
        {
            warnings.push("💡 已自动启用 show_in_prompt（notification_mode=prompt）".to_string());
            self.likan.show_in_prompt = true;
        }

        warnings
    }
}
```

---

## 📝 配置文件简化

### 方案1: 分层配置文件

```bash
~/.realconsole/
  ├── config.minimal.yaml   # 最小配置模板
  ├── config.standard.yaml  # 标准配置模板
  ├── config.full.yaml      # 完整配置模板
  └── config.yaml           # 用户配置（优先）
```

**首次运行**:
```bash
$ realconsole
⚠️ 未找到配置文件

选择配置模板：
  1. 最小配置（推荐新手）- 只需 API key
  2. 标准配置（推荐）- 常用功能
  3. 完整配置（高级用户）- 全部选项

请选择 [1/2/3]: 1

✅ 已创建 ~/.realconsole/config.yaml
💡 请设置环境变量：export DEEPSEEK_API_KEY="your-key"
```

### 方案2: 向导式配置

```bash
$ realconsole wizard

🧙 RealConsole 配置向导
━━━━━━━━━━━━━━━━━━━━━━

1️⃣ LLM 配置
   使用哪个 LLM 服务？
   [1] Deepseek（推荐，性价比高）
   [2] OpenAI
   [3] 本地 Ollama
   选择 [1/2/3]: 1

   API Key: ****************
   ✅ 已保存

2️⃣ 功能配置
   启用自主学习（离坎炼化炉）？[Y/n]: Y
   ✅ 已启用，将每 5 分钟自动优化

   在提示符显示状态？[y/N]: N
   ✅ 使用最小通知模式

3️⃣ 完成！
   配置已保存到 ~/.realconsole/config.yaml

   运行 `realconsole` 开始使用 🚀
```

---

## 🎯 用户体验优化

### 1. 友好的错误提示

**当前**:
```
Error: Failed to load config: No such file or directory
```

**优化后**:
```
💡 首次运行？未找到配置文件

快速开始：
  1. 运行配置向导：realconsole wizard
  2. 使用环境变量：export DEEPSEEK_API_KEY="your-key"
  3. 或创建配置文件：cp ~/.realconsole/config.minimal.yaml ~/.realconsole/config.yaml

需要帮助？运行 `realconsole --help`
```

### 2. 自动诊断

```rust
impl Config {
    pub fn diagnose(&self) -> DiagnosticReport {
        let mut report = DiagnosticReport::new();

        // 检查 API key
        if self.llm.api_key.is_empty() {
            report.add_warning(
                "LLM API key 未配置",
                "设置环境变量 DEEPSEEK_API_KEY 或运行 `realconsole wizard`"
            );
        }

        // 检查网络
        if !Self::check_network(&self.llm.endpoint) {
            report.add_error(
                "无法连接 LLM 服务",
                format!("请检查网络或端点: {}", self.llm.endpoint)
            );
        }

        // 检查本地存储
        if !Self::check_storage_writable() {
            report.add_warning(
                "无法写入本地存储",
                "反馈系统和历史记录可能无法保存"
            );
        }

        report
    }
}
```

### 3. 智能迁移

```rust
/// 从旧版本配置迁移
impl Config {
    pub fn migrate_from_v1_8_2(old_config: &str) -> Result<Self> {
        // 解析旧配置
        let old: OldConfig = serde_yml::from_str(old_config)?;

        // 映射到新配置
        let new = Self {
            llm: LlmConfig::from_old(&old.llm),
            likan: LiKanConfig::from_old(&old.likan),
            // ... 其他映射
        };

        // 添加新默认值
        new.fill_new_defaults();

        Ok(new)
    }
}
```

---

## 📊 预期效果

### 配置简化对比

| 场景 | 配置行数 | 优化前 | 优化后 | 提升 |
|------|---------|--------|--------|------|
| 新手启动 | 必填项 | ~20行 | 1行 | **95%** |
| 普通使用 | 常用配置 | ~50行 | 3-5行 | **90%** |
| 高级定制 | 全部配置 | ~150行 | ~150行 | 0% |

### 用户体验提升

| 指标 | 优化前 | 优化后 | 提升 |
|------|--------|--------|------|
| 首次配置时间 | 5-10分钟 | 30秒 | **90%** |
| 配置错误率 | ~30% | < 5% | **83%** |
| 上手难度 | 中 | 低 | ✅ |
| 满意度 | 3.5/5 | 4.8/5 | +37% |

---

## 🚀 实施计划

### Phase 1: 基础智能默认（本周）
- [ ] Config::smart_defaults() 实现
- [ ] 环境变量智能检测
- [ ] 配置验证与修复
- [ ] 最小配置模板

### Phase 2: 向导与诊断（下周）
- [ ] realconsole wizard 改进
- [ ] 配置诊断系统
- [ ] 友好错误提示
- [ ] 配置迁移工具

### Phase 3: 自适应优化（本月）
- [ ] 使用模式检测
- [ ] 动态参数调整
- [ ] 智能推荐配置
- [ ] A/B 测试优化

---

## 📝 文档更新

### 新增文档
- [ ] 快速开始指南（5分钟上手）
- [ ] 配置最佳实践
- [ ] 常见问题 FAQ

### 更新文档
- [ ] README（强调零配置）
- [ ] 用户指南（简化配置部分）
- [ ] 开发文档（配置系统架构）

---

**设计原则**: Less Config, More Magic
**目标**: 新用户 30 秒启动，老用户完全掌控
**下一步**: 立即实施 Phase 1 🚀
