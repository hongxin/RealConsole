//! 内置意图库
//!
//! **设计哲学**：少则得，多则惑（道德经第二十二章）
//!
//! 本模块提供精选的 16 个高频意图和模板，涵盖日常使用的 80% 场景。
//!
//! ## 四大领域
//!
//! - **FileOps**: 文件操作（查找、统计、分析）
//! - **DataOps**: 数据处理（搜索、过滤、排序）
//! - **DiagnosticOps**: 诊断分析（错误分析、健康检查）
//! - **SystemOps**: 系统管理（进程、磁盘、网络）
//!
//! ## 使用示例
//!
//! ```rust
//! use simpleconsole::dsl::intent::builtin::BuiltinIntents;
//!
//! let builtin = BuiltinIntents::new();
//!
//! // 获取所有内置意图
//! let intents = builtin.all_intents();
//!
//! // 获取所有内置模板
//! let templates = builtin.all_templates();
//! ```

use crate::dsl::intent::{EntityType, Intent, IntentDomain, IntentMatcher, Template, TemplateEngine};

/// 内置意图和模板集合
///
/// BuiltinIntents 提供开箱即用的意图识别和命令生成能力。
///
/// # 设计原则
///
/// 1. **精选而非全面** - 只提供最常用的意图
/// 2. **简单而非复杂** - 每个意图都有清晰的语义
/// 3. **可扩展** - 用户可以添加自定义意图
///
/// # 示例
///
/// ```rust
/// use simpleconsole::dsl::intent::builtin::BuiltinIntents;
///
/// let builtin = BuiltinIntents::new();
///
/// // 初始化匹配器
/// let mut matcher = builtin.create_matcher();
///
/// // 匹配用户输入
/// let matches = matcher.match_intent("统计 Python 代码行数");
/// if let Some(m) = matches.first() {
///     println!("匹配到意图: {}", m.intent.name);
/// }
///
/// // 初始化模板引擎
/// let mut engine = builtin.create_engine();
/// ```
#[derive(Debug)]
pub struct BuiltinIntents;

impl BuiltinIntents {
    /// 创建内置意图集合
    pub fn new() -> Self {
        Self
    }

    /// 获取所有内置意图
    ///
    /// # 返回
    ///
    /// 包含 24 个预定义意图的向量
    pub fn all_intents(&self) -> Vec<Intent> {
        vec![
            // ===== 文件操作类 (FileOps) =====
            self.count_python_lines(),
            self.count_files(),
            self.find_files_by_size(),      // Phase 6.2: 按体积查找（支持最大/最小）
            self.find_recent_files(),
            self.find_files_by_name(),     // Phase 6.1: 按名称查找文件
            self.list_directory(),
            self.create_directory(),        // Phase 6.1: 创建目录
            self.create_symlink(),          // Phase 6.1: 创建符号链接
            // ===== 数据处理类 (DataOps) =====
            self.grep_pattern(),
            self.sort_lines(),
            self.count_pattern(),
            self.count_file_stats(),        // Phase 6.1: 统计文件信息
            self.compare_files(),           // Phase 6.1: 比较文件差异
            // ===== 诊断分析类 (DiagnosticOps) =====
            self.analyze_errors(),
            self.check_disk_usage(),
            self.view_system_logs(),
            // ===== 系统管理类 (SystemOps) =====
            self.list_processes(),
            self.check_memory_usage(),
            self.check_cpu_usage(),
            self.check_network_connections(),
            self.check_uptime(),
            self.ping_host(),               // Phase 6.1: 测试网络连通性
            self.view_env_var(),            // Phase 6.1: 查看环境变量
            self.check_service_status(),    // Phase 6.1: 查看服务状态
        ]
    }

    /// 获取所有内置模板
    ///
    /// # 返回
    ///
    /// 包含 24 个预定义模板的向量
    pub fn all_templates(&self) -> Vec<Template> {
        vec![
            // ===== 文件操作类 =====
            self.template_count_python_lines(),
            self.template_count_files(),
            self.template_find_files_by_size(),     // Phase 6.2: 按体积查找
            self.template_find_recent_files(),
            self.template_find_files_by_name(),     // Phase 6.1
            self.template_list_directory(),
            self.template_create_directory(),        // Phase 6.1
            self.template_create_symlink(),          // Phase 6.1
            // ===== 数据处理类 =====
            self.template_grep_pattern(),
            self.template_sort_lines(),
            self.template_count_pattern(),
            self.template_count_file_stats(),        // Phase 6.1
            self.template_compare_files(),           // Phase 6.1
            // ===== 诊断分析类 =====
            self.template_analyze_errors(),
            self.template_check_disk_usage(),
            self.template_view_system_logs(),
            // ===== 系统管理类 =====
            self.template_list_processes(),
            self.template_check_memory_usage(),
            self.template_check_cpu_usage(),
            self.template_check_network_connections(),
            self.template_check_uptime(),
            self.template_ping_host(),               // Phase 6.1
            self.template_view_env_var(),            // Phase 6.1
            self.template_check_service_status(),    // Phase 6.1
        ]
    }

    /// 创建预加载的意图匹配器
    ///
    /// # 返回
    ///
    /// 包含所有内置意图的 IntentMatcher
    pub fn create_matcher(&self) -> IntentMatcher {
        let mut matcher = IntentMatcher::new();
        for intent in self.all_intents() {
            matcher.register(intent);
        }
        matcher
    }

    /// 创建预加载的模板引擎
    ///
    /// # 返回
    ///
    /// 包含所有内置模板的 TemplateEngine
    pub fn create_engine(&self) -> TemplateEngine {
        let mut engine = TemplateEngine::new();
        for template in self.all_templates() {
            engine.register(template);
        }
        engine
    }

    // ==================== 文件操作类意图 ====================

    /// 意图: 统计 Python 代码行数
    ///
    /// **关键词**: python, 行数, 统计, 代码
    /// **模式**: 统计.*python.*行数
    /// **实体**: path (路径)
    fn count_python_lines(&self) -> Intent {
        Intent::new(
            "count_python_lines",
            IntentDomain::FileOps,
            vec![
                "python".to_string(),
                "行数".to_string(),
                "统计".to_string(),
                "代码".to_string(),
            ],
            vec![r"(?i)统计.*python.*行数".to_string()],
            0.5,
        )
        .with_entity("path", EntityType::Path(".".to_string()))
    }

    /// 模板: 统计 Python 代码行数
    fn template_count_python_lines(&self) -> Template {
        Template::new(
            "count_python_lines",
            "find {path} -name '*.py' -type f -exec wc -l {} + | tail -1",
            vec!["path".to_string()],
        )
        .with_description("统计指定目录下所有 Python 文件的总行数")
    }

    /// 意图: 统计文件数量
    ///
    /// **关键词**: 统计, 文件, 数量, 个数
    /// **模式**: 统计.*文件.*(数量|个数)
    /// **实体**: path (路径), ext (文件类型)
    fn count_files(&self) -> Intent {
        Intent::new(
            "count_files",
            IntentDomain::FileOps,
            vec![
                "统计".to_string(),
                "文件".to_string(),
                "数量".to_string(),
                "个数".to_string(),
            ],
            vec![r"(?i)统计.*文件.*(数量|个数)".to_string()],
            0.5,
        )
        .with_entity("path", EntityType::Path(".".to_string()))
        .with_entity("ext", EntityType::FileType("*".to_string()))
    }

    /// 模板: 统计文件数量
    fn template_count_files(&self) -> Template {
        Template::new(
            "count_files",
            "find {path} -name '*.{ext}' -type f | wc -l",
            vec!["path".to_string(), "ext".to_string()],
        )
        .with_description("统计指定目录下特定类型的文件数量")
    }

    /// 意图: 按体积查找文件（支持最大/最小）
    ///
    /// **关键词**: 查找, 显示, 大文件, 文件, 大于, 体积, 最大, 最小
    /// **模式**: (查找|显示).*(大文件|大于|体积|最大|最小)
    /// **实体**: path (路径), ext (文件类型), limit (显示数量), sort_order (排序方向)
    /// **哲学**: 巽卦（过滤）+ 兑卦（条件），体现"变"的智慧
    ///          - 象（不变）：查找文件并排序
    ///          - 爻（变化）：sort_order 从 "-hr" (最大) 变为 "-h" (最小)
    ///
    /// **Phase 6.2.1**: 参数化模板，从静态到动态的第一步
    fn find_files_by_size(&self) -> Intent {
        Intent::new(
            "find_files_by_size",
            IntentDomain::FileOps,
            vec![
                "查找".to_string(),
                "显示".to_string(),
                "大文件".to_string(),
                "小文件".to_string(),
                "文件".to_string(),
                "大于".to_string(),
                "小于".to_string(),
                "体积".to_string(),
                "最大".to_string(),
                "最小".to_string(),
            ],
            vec![
                r"(?i)(查找|显示|列出).*(大文件|大于|小文件|小于)".to_string(),
                r"(?i)(体积|大小).*(最大|最小|大于|小于)".to_string(),
                r"(?i)(最大|最小).*(文件|file)".to_string(),
            ],
            0.7,  // 提高置信度阈值，优先于 list_directory
        )
        .with_entity("path", EntityType::Path(".".to_string()))
        .with_entity("ext", EntityType::FileType("*".to_string()))
        .with_entity("limit", EntityType::Number(10.0))
        // sort_order 将由实体提取器动态确定："-hr" (降序/最大) 或 "-h" (升序/最小)
        .with_entity("sort_order", EntityType::Custom("sort".to_string(), "-hr".to_string()))
    }

    /// 模板: 按体积查找文件
    ///
    /// **参数化设计**：
    /// - `sort_order`: "-hr" (降序，显示最大) 或 "-h" (升序，显示最小)
    /// - 体现"象"（查找+排序）不变，"爻"（方向）可变的哲学
    fn template_find_files_by_size(&self) -> Template {
        Template::new(
            "find_files_by_size",
            "find {path} -name '*.{ext}' -type f -exec ls -lh {} + | sort -k5 {sort_order} | head -n {limit}",
            vec!["path".to_string(), "ext".to_string(), "sort_order".to_string(), "limit".to_string()],
        )
        .with_description("按体积查找文件（支持最大/最小，可指定文件类型）")
    }

    /// 意图: 查找最近修改的文件
    ///
    /// **关键词**: 查找, 显示, 最近, 修改, 更新, 文件, 新, latest
    /// **模式**:
    ///   - (查找|显示|列出).*(最近|最新)
    ///   - (最近|最新).*(修改|更新|变更).*文件
    ///   - 文件.*(最近|最新)
    /// **实体**: path (路径), ext (文件类型), limit (显示数量)
    fn find_recent_files(&self) -> Intent {
        Intent::new(
            "find_recent_files",
            IntentDomain::FileOps,
            vec![
                "查找".to_string(),
                "显示".to_string(),
                "列出".to_string(),
                "最近".to_string(),
                "最新".to_string(),
                "修改".to_string(),
                "更新".to_string(),
                "文件".to_string(),
            ],
            vec![
                r"(?i)(查找|显示|列出).*(最近|最新)".to_string(),
                r"(?i)(最近|最新).*(修改|更新|变更).*文件".to_string(),
                r"(?i)文件.*(最近|最新)".to_string(),
            ],
            0.65,  // 提高优先级，避免被 list_directory 覆盖
        )
        .with_entity("path", EntityType::Path(".".to_string()))
        .with_entity("ext", EntityType::FileType("*".to_string()))
        .with_entity("limit", EntityType::Number(10.0))
    }

    /// 模板: 查找最近修改的文件
    fn template_find_recent_files(&self) -> Template {
        Template::new(
            "find_recent_files",
            "find {path} -name '*.{ext}' -type f -exec ls -lt {} + | head -n {limit}",
            vec!["path".to_string(), "ext".to_string(), "limit".to_string()],
        )
        .with_description("查找指定目录下最近修改的文件（按时间排序，支持文件类型过滤）")
    }

    // ==================== 数据处理类意图 ====================

    /// 意图: 搜索文本模式
    ///
    /// **关键词**: 搜索, grep, 查找, 匹配
    /// **模式**: (搜索|查找).*模式
    fn grep_pattern(&self) -> Intent {
        Intent::new(
            "grep_pattern",
            IntentDomain::DataOps,
            vec![
                "搜索".to_string(),
                "grep".to_string(),
                "查找".to_string(),
                "匹配".to_string(),
            ],
            vec![r"(?i)(搜索|查找).*模式".to_string()],
            0.5,
        )
    }

    /// 模板: 搜索文本模式
    fn template_grep_pattern(&self) -> Template {
        Template::new(
            "grep_pattern",
            "grep -r '{pattern}' {path}",
            vec!["pattern".to_string(), "path".to_string()],
        )
        .with_description("在指定目录下递归搜索文本模式")
    }

    /// 意图: 排序文本行
    ///
    /// **关键词**: 排序, sort, 排列
    /// **模式**: 排序.*文本
    fn sort_lines(&self) -> Intent {
        Intent::new(
            "sort_lines",
            IntentDomain::DataOps,
            vec![
                "排序".to_string(),
                "sort".to_string(),
                "排列".to_string(),
            ],
            vec![r"(?i)排序.*文本".to_string()],
            0.5,
        )
    }

    /// 模板: 排序文本行
    fn template_sort_lines(&self) -> Template {
        Template::new(
            "sort_lines",
            "sort {file}",
            vec!["file".to_string()],
        )
        .with_description("对文件内容进行排序")
    }

    /// 意图: 统计模式出现次数
    ///
    /// **关键词**: 统计, 次数, 出现, 模式
    /// **模式**: 统计.*(次数|出现)
    fn count_pattern(&self) -> Intent {
        Intent::new(
            "count_pattern",
            IntentDomain::DataOps,
            vec![
                "统计".to_string(),
                "次数".to_string(),
                "出现".to_string(),
                "模式".to_string(),
            ],
            vec![r"(?i)统计.*(次数|出现)".to_string()],
            0.5,
        )
    }

    /// 模板: 统计模式出现次数
    fn template_count_pattern(&self) -> Template {
        Template::new(
            "count_pattern",
            "grep -c '{pattern}' {file}",
            vec!["pattern".to_string(), "file".to_string()],
        )
        .with_description("统计文件中匹配模式的行数")
    }

    // ==================== 诊断分析类意图 ====================

    /// 意图: 分析错误日志
    ///
    /// **关键词**: 分析, 错误, error, 日志
    /// **模式**: 分析.*错误
    fn analyze_errors(&self) -> Intent {
        Intent::new(
            "analyze_errors",
            IntentDomain::DiagnosticOps,
            vec![
                "分析".to_string(),
                "错误".to_string(),
                "error".to_string(),
                "日志".to_string(),
            ],
            vec![r"(?i)分析.*错误".to_string()],
            0.5,
        )
    }

    /// 模板: 分析错误日志
    fn template_analyze_errors(&self) -> Template {
        Template::new(
            "analyze_errors",
            "grep -i 'error' {file} | sort | uniq -c | sort -nr",
            vec!["file".to_string()],
        )
        .with_description("统计日志文件中各类错误的出现次数")
    }

    /// 意图: 检查磁盘使用情况
    ///
    /// **关键词**: 检查, 磁盘, 空间, 使用
    /// **模式**: 检查.*磁盘
    /// **实体**: path (路径), limit (显示数量)
    fn check_disk_usage(&self) -> Intent {
        Intent::new(
            "check_disk_usage",
            IntentDomain::DiagnosticOps,
            vec![
                "检查".to_string(),
                "磁盘".to_string(),
                "空间".to_string(),
                "使用".to_string(),
            ],
            vec![r"(?i)检查.*磁盘".to_string()],
            0.5,
        )
        .with_entity("path", EntityType::Path(".".to_string()))
        .with_entity("limit", EntityType::Number(10.0))
    }

    /// 模板: 检查磁盘使用情况
    fn template_check_disk_usage(&self) -> Template {
        Template::new(
            "check_disk_usage",
            "du -sh {path}/* | sort -hr | head -n {limit}",
            vec!["path".to_string(), "limit".to_string()],
        )
        .with_description("显示指定目录下占用空间最多的前 N 个文件/目录")
    }

    // ==================== 系统管理类意图 ====================

    /// 意图: 列出进程
    ///
    /// **关键词**: 列出, 进程, ps, process
    /// **模式**: 列出.*进程
    fn list_processes(&self) -> Intent {
        Intent::new(
            "list_processes",
            IntentDomain::SystemOps,
            vec![
                "列出".to_string(),
                "进程".to_string(),
                "ps".to_string(),
                "process".to_string(),
            ],
            vec![r"(?i)列出.*进程".to_string()],
            0.5,
        )
    }

    /// 模板: 列出进程
    fn template_list_processes(&self) -> Template {
        Template::new(
            "list_processes",
            "ps aux | grep '{name}' | grep -v grep",
            vec!["name".to_string()],
        )
        .with_description("列出包含指定名称的进程")
    }

    /// 意图: 检查内存使用情况
    ///
    /// **关键词**: 检查, 内存, memory, RAM, 可用
    /// **模式**: (检查|查看).*(内存|memory|ram)
    fn check_memory_usage(&self) -> Intent {
        Intent::new(
            "check_memory_usage",
            IntentDomain::SystemOps,
            vec![
                "检查".to_string(),
                "查看".to_string(),
                "内存".to_string(),
                "memory".to_string(),
                "RAM".to_string(),
                "可用".to_string(),
            ],
            vec![
                r"(?i)(检查|查看).*(内存|memory|ram)".to_string(),
                r"(?i)内存.*(使用|情况|可用)".to_string(),
            ],
            0.6,  // 稍高的置信度阈值，避免误匹配
        )
    }

    /// 模板: 检查内存使用情况
    fn template_check_memory_usage(&self) -> Template {
        Template::new(
            "check_memory_usage",
            "top -l 1 | head -n 10 | grep PhysMem",
            vec![],  // 无需参数
        )
        .with_description("查看系统内存使用情况（macOS）")
    }

    // ==================== Phase 1 扩展: 高优先级意图 ====================

    /// 意图: 查看目录内容
    ///
    /// **关键词**: 查看, 列出, 目录, 文件, ls, list [移除"显示"，避免与过滤型Intent冲突]
    /// **模式**: (查看|列出).*(目录|文件夹|当前)
    /// **用例**: #1 查看当前目录下的文件和子目录
    /// **哲学**: 降低优先级，让过滤型Intent优先匹配（一分为三：查看vs过滤vs排序）
    fn list_directory(&self) -> Intent {
        Intent::new(
            "list_directory",
            IntentDomain::FileOps,
            vec![
                "查看".to_string(),
                // 移除 "显示" - 让 find_large_files 等过滤型Intent优先
                "列出".to_string(),
                "目录".to_string(),
                "文件".to_string(),
                "ls".to_string(),
            ],
            vec![
                // 简化正则，不使用否定前瞻断言（Rust regex不支持）
                r"(?i)(查看|列出).*(目录|文件夹|当前)".to_string(),
                r"(?i)(ls|list).*(dir|directory|files?)".to_string(),
                r"(?i)^ls\s*$".to_string(),  // 单独的 "ls" 命令
            ],
            0.50,  // 降低置信度阈值，让过滤型Intent优先
        )
        .with_entity("path", EntityType::Path(".".to_string()))
    }

    /// 模板: 查看目录内容
    fn template_list_directory(&self) -> Template {
        Template::new(
            "list_directory",
            "ls -lh {path}",
            vec!["path".to_string()],
        )
        .with_description("列出指定目录下的文件和子目录")
    }

    /// 意图: 检查CPU使用率
    ///
    /// **关键词**: 检查, 查看, CPU, 使用率, 负载, load
    /// **模式**: (检查|查看).*(CPU|cpu|负载|load)
    /// **用例**: #10 检查系统负载和CPU使用率
    fn check_cpu_usage(&self) -> Intent {
        Intent::new(
            "check_cpu_usage",
            IntentDomain::SystemOps,
            vec![
                "检查".to_string(),
                "查看".to_string(),
                "CPU".to_string(),
                "cpu".to_string(),
                "使用率".to_string(),
                "负载".to_string(),
            ],
            vec![
                r"(?i)(检查|查看).*(CPU|cpu|负载|load)".to_string(),
                r"(?i)(CPU|cpu).*(使用|占用|情况)".to_string(),
            ],
            0.6,
        )
    }

    /// 模板: 检查CPU使用率
    fn template_check_cpu_usage(&self) -> Template {
        Template::new(
            "check_cpu_usage",
            "uptime && top -l 1 | head -n 10 | grep -E '(CPU|Load)'",
            vec![],
        )
        .with_description("查看系统负载和CPU使用率（macOS）")
    }

    /// 意图: 检查网络连接
    ///
    /// **关键词**: 网络, 连接, 端口, 监听, netstat, socket
    /// **模式**: (检查|查看).*(网络|连接|端口)
    /// **用例**: #7 检查网络连接状态和端口监听
    fn check_network_connections(&self) -> Intent {
        Intent::new(
            "check_network_connections",
            IntentDomain::SystemOps,
            vec![
                "网络".to_string(),
                "连接".to_string(),
                "端口".to_string(),
                "监听".to_string(),
                "netstat".to_string(),
            ],
            vec![
                r"(?i)(检查|查看).*(网络|连接|端口)".to_string(),
                r"(?i)(netstat|lsof|socket).*(端口|port|listen)".to_string(),
            ],
            0.6,
        )
    }

    /// 模板: 检查网络连接
    fn template_check_network_connections(&self) -> Template {
        Template::new(
            "check_network_connections",
            "netstat -an | grep LISTEN | head -n 20",
            vec![],
        )
        .with_description("显示系统正在监听的网络端口")
    }

    /// 意图: 查看系统日志
    ///
    /// **关键词**: 日志, log, 查看, 系统, 错误, 警告
    /// **模式**: (查看|显示).*(日志|log)
    /// **用例**: #6 查看系统日志文件的最新内容
    fn view_system_logs(&self) -> Intent {
        Intent::new(
            "view_system_logs",
            IntentDomain::DiagnosticOps,
            vec![
                "日志".to_string(),
                "log".to_string(),
                "查看".to_string(),
                "系统".to_string(),
                "错误".to_string(),
            ],
            vec![
                r"(?i)(查看|显示).*(日志|log)".to_string(),
                r"(?i)(系统|system).*(日志|log|错误)".to_string(),
            ],
            0.6,
        )
        .with_entity("lines", EntityType::Number(50.0))
    }

    /// 模板: 查看系统日志
    fn template_view_system_logs(&self) -> Template {
        Template::new(
            "view_system_logs",
            "log show --predicate 'eventMessage contains \"error\" OR eventMessage contains \"fail\"' --info --last 1h | tail -n {lines}",
            vec!["lines".to_string()],
        )
        .with_description("查看系统日志中的错误和失败信息（macOS）")
    }

    /// 意图: 查看系统运行时长
    ///
    /// **关键词**: 运行时长, 启动时间, uptime, 多久, 开机
    /// **模式**: (系统|机器).*(运行|启动|开机).*(时间|多久)
    /// **用例**: #9 查看系统启动时间和运行时长
    fn check_uptime(&self) -> Intent {
        Intent::new(
            "check_uptime",
            IntentDomain::SystemOps,
            vec![
                "运行时长".to_string(),
                "启动时间".to_string(),
                "uptime".to_string(),
                "多久".to_string(),
                "开机".to_string(),
            ],
            vec![
                r"(?i)(系统|机器).*(运行|启动|开机).*(时间|多久)".to_string(),
                r"(?i)(uptime|运行时长|开机时间)".to_string(),
            ],
            0.55,
        )
    }

    /// 模板: 查看系统运行时长
    fn template_check_uptime(&self) -> Template {
        Template::new(
            "check_uptime",
            "uptime",
            vec![],
        )
        .with_description("显示系统启动时间和运行时长")
    }

    // ==================== Phase 6.1: 安全功能增强 ====================

    /// 意图: 按名称查找文件
    ///
    /// **关键词**: 查找, 搜索, 寻找, 文件, 名字, 名称, find
    /// **模式**: (查找|搜索|寻找).*(文件|file).*名(字|称)
    /// **用例**: #11 查找文件按名称或类型
    fn find_files_by_name(&self) -> Intent {
        Intent::new(
            "find_files_by_name",
            IntentDomain::FileOps,
            vec![
                "查找".to_string(),
                "搜索".to_string(),
                "寻找".to_string(),
                "文件".to_string(),
                "名字".to_string(),
                "名称".to_string(),
                "find".to_string(),
            ],
            vec![
                r"(?i)(查找|搜索|寻找).*(文件|file).*名(字|称)".to_string(),
                r"(?i)(find|locate).*(file|name)".to_string(),
                r"(?i)名(字|称).*为.*的.*文件".to_string(),
            ],
            0.6,
        )
        .with_entity("path", EntityType::Path(".".to_string()))
    }

    /// 模板: 按名称查找文件
    fn template_find_files_by_name(&self) -> Template {
        Template::new(
            "find_files_by_name",
            "find {path} -name '{name}' -type f",
            vec!["path".to_string(), "name".to_string()],
        )
        .with_description("在指定目录下按名称查找文件")
    }

    /// 意图: 统计文件行数/字数
    ///
    /// **关键词**: 统计, 计算, 行数, 字数, 单词, wc
    /// **模式**: 统计.*(行数|字数|单词)
    /// **用例**: #42 统计文件行数、字数
    fn count_file_stats(&self) -> Intent {
        Intent::new(
            "count_file_stats",
            IntentDomain::DataOps,
            vec![
                "统计".to_string(),
                "计算".to_string(),
                "行数".to_string(),
                "字数".to_string(),
                "单词".to_string(),
                "wc".to_string(),
            ],
            vec![
                r"(?i)统计.*(行数|字数|单词)".to_string(),
                r"(?i)(行数|字数|单词).*统计".to_string(),
                r"(?i)wc.*file".to_string(),
            ],
            0.6,
        )
        .with_entity("file", EntityType::Path("".to_string()))
    }

    /// 模板: 统计文件行数/字数
    fn template_count_file_stats(&self) -> Template {
        Template::new(
            "count_file_stats",
            "wc {file}",
            vec!["file".to_string()],
        )
        .with_description("统计文件的行数、单词数和字节数")
    }

    /// 意图: 比较文件差异
    ///
    /// **关键词**: 比较, 对比, 差异, 不同, diff
    /// **模式**: 比较.*(文件|file)
    /// **用例**: #41 比较两个文件的差异
    fn compare_files(&self) -> Intent {
        Intent::new(
            "compare_files",
            IntentDomain::DataOps,
            vec![
                "比较".to_string(),
                "对比".to_string(),
                "差异".to_string(),
                "不同".to_string(),
                "diff".to_string(),
            ],
            vec![
                r"(?i)比较.*(文件|file)".to_string(),
                r"(?i)(差异|不同|区别)".to_string(),
                r"(?i)diff.*file".to_string(),
            ],
            0.6,
        )
        .with_entity("file1", EntityType::Path("".to_string()))
        .with_entity("file2", EntityType::Path("".to_string()))
    }

    /// 模板: 比较文件差异
    fn template_compare_files(&self) -> Template {
        Template::new(
            "compare_files",
            "diff -u {file1} {file2}",
            vec!["file1".to_string(), "file2".to_string()],
        )
        .with_description("比较两个文件的差异（统一格式）")
    }

    /// 意图: 测试网络连通性
    ///
    /// **关键词**: ping, 测试, 检测, 网络, 连通, 可达
    /// **模式**: ping.*host
    /// **用例**: #20 测试网络连通性
    fn ping_host(&self) -> Intent {
        Intent::new(
            "ping_host",
            IntentDomain::SystemOps,
            vec![
                "ping".to_string(),
                "测试".to_string(),
                "检测".to_string(),
                "网络".to_string(),
                "连通".to_string(),
                "可达".to_string(),
            ],
            vec![
                r"(?i)ping.*host".to_string(),
                r"(?i)测试.*(网络|连通)".to_string(),
                r"(?i)(检测|测试).*(可达|连接)".to_string(),
            ],
            0.65,
        )
        .with_entity("count", EntityType::Number(4.0))
    }

    /// 模板: 测试网络连通性
    fn template_ping_host(&self) -> Template {
        Template::new(
            "ping_host",
            "ping -c {count} {host}",
            vec!["host".to_string(), "count".to_string()],
        )
        .with_description("测试指定主机的网络连通性")
    }

    /// 意图: 查看环境变量
    ///
    /// **关键词**: 查看, 显示, 环境变量, 变量, env, echo
    /// **模式**: (查看|显示).*(环境变量|变量)
    /// **用例**: #39 查看环境变量
    fn view_env_var(&self) -> Intent {
        Intent::new(
            "view_env_var",
            IntentDomain::SystemOps,
            vec![
                "查看".to_string(),
                "显示".to_string(),
                "环境变量".to_string(),
                "变量".to_string(),
                "env".to_string(),
                "echo".to_string(),
            ],
            vec![
                r"(?i)(查看|显示).*(环境变量|变量)".to_string(),
                r"(?i)env.*var".to_string(),
                r"(?i)echo.*\$".to_string(),
            ],
            0.6,
        )
    }

    /// 模板: 查看环境变量
    fn template_view_env_var(&self) -> Template {
        Template::new(
            "view_env_var",
            "echo ${var}",
            vec!["var".to_string()],
        )
        .with_description("查看指定环境变量的值")
    }

    /// 意图: 查看服务状态
    ///
    /// **关键词**: 查看, 检查, 服务, 状态, 运行, service
    /// **模式**: (查看|检查).*(服务|service)
    /// **用例**: #22 查看服务状态
    fn check_service_status(&self) -> Intent {
        Intent::new(
            "check_service_status",
            IntentDomain::SystemOps,
            vec![
                "查看".to_string(),
                "检查".to_string(),
                "服务".to_string(),
                "状态".to_string(),
                "运行".to_string(),
                "service".to_string(),
            ],
            vec![
                r"(?i)(查看|检查).*(服务|service)".to_string(),
                r"(?i)服务.*状态".to_string(),
                r"(?i)(systemctl|launchctl).*status".to_string(),
            ],
            0.6,
        )
    }

    /// 模板: 查看服务状态
    fn template_check_service_status(&self) -> Template {
        // 跨平台支持：macOS 使用 launchctl, Linux 使用 systemctl
        #[cfg(target_os = "macos")]
        let command = "launchctl list | grep {service}";

        #[cfg(not(target_os = "macos"))]
        let command = "systemctl status {service}";

        Template::new(
            "check_service_status",
            command,
            vec!["service".to_string()],
        )
        .with_description("查看指定服务的运行状态")
    }

    /// 意图: 创建目录
    ///
    /// **关键词**: 创建, 新建, 建立, 目录, 文件夹, mkdir
    /// **模式**: (创建|新建|建立).*(目录|文件夹|folder)
    /// **用例**: #4 创建新的目录
    fn create_directory(&self) -> Intent {
        Intent::new(
            "create_directory",
            IntentDomain::FileOps,
            vec![
                "创建".to_string(),
                "新建".to_string(),
                "建立".to_string(),
                "目录".to_string(),
                "文件夹".to_string(),
                "mkdir".to_string(),
            ],
            vec![
                r"(?i)(创建|新建|建立).*(目录|文件夹|folder)".to_string(),
                r"(?i)mkdir.*dir".to_string(),
            ],
            0.65,
        )
        .with_entity("path", EntityType::Path("".to_string()))
    }

    /// 模板: 创建目录
    fn template_create_directory(&self) -> Template {
        Template::new(
            "create_directory",
            "mkdir -p {path}",
            vec!["path".to_string()],
        )
        .with_description("创建目录（自动创建父目录）")
    }

    /// 意图: 创建符号链接
    ///
    /// **关键词**: 创建, 建立, 符号链接, 软链接, 链接, ln
    /// **模式**: (创建|建立).*(符号链接|软链接|symlink)
    /// **用例**: #40 创建符号链接
    fn create_symlink(&self) -> Intent {
        Intent::new(
            "create_symlink",
            IntentDomain::FileOps,
            vec![
                "创建".to_string(),
                "建立".to_string(),
                "符号链接".to_string(),
                "软链接".to_string(),
                "链接".to_string(),
                "ln".to_string(),
            ],
            vec![
                r"(?i)(创建|建立).*(符号链接|软链接|symlink)".to_string(),
                r"(?i)ln.*-s".to_string(),
            ],
            0.65,
        )
        .with_entity("source", EntityType::Path("".to_string()))
        .with_entity("target", EntityType::Path("".to_string()))
    }

    /// 模板: 创建符号链接
    fn template_create_symlink(&self) -> Template {
        Template::new(
            "create_symlink",
            "ln -s {source} {target}",
            vec!["source".to_string(), "target".to_string()],
        )
        .with_description("创建符号链接")
    }
}

impl Default for BuiltinIntents {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_builtin_creation() {
        let builtin = BuiltinIntents::new();
        assert_eq!(builtin.all_intents().len(), 24);
        assert_eq!(builtin.all_templates().len(), 24);
    }

    #[test]
    fn test_all_intent_names() {
        let builtin = BuiltinIntents::new();
        let intents = builtin.all_intents();

        let names: Vec<String> = intents.iter().map(|i| i.name.clone()).collect();

        assert!(names.contains(&"count_python_lines".to_string()));
        assert!(names.contains(&"count_files".to_string()));
        assert!(names.contains(&"find_files_by_size".to_string()));
        assert!(names.contains(&"find_recent_files".to_string()));
        assert!(names.contains(&"list_directory".to_string()));
        assert!(names.contains(&"grep_pattern".to_string()));
        assert!(names.contains(&"sort_lines".to_string()));
        assert!(names.contains(&"count_pattern".to_string()));
        assert!(names.contains(&"analyze_errors".to_string()));
        assert!(names.contains(&"check_disk_usage".to_string()));
        assert!(names.contains(&"view_system_logs".to_string()));
        assert!(names.contains(&"list_processes".to_string()));
        assert!(names.contains(&"check_memory_usage".to_string()));
        assert!(names.contains(&"check_cpu_usage".to_string()));
        assert!(names.contains(&"check_network_connections".to_string()));
        assert!(names.contains(&"check_uptime".to_string()));
    }

    #[test]
    fn test_intent_domains() {
        let builtin = BuiltinIntents::new();
        let intents = builtin.all_intents();

        // FileOps (8 个): count_python_lines, count_files, find_large_files, find_recent_files,
        //                 find_files_by_name, list_directory, create_directory, create_symlink
        assert_eq!(intents[0].domain, IntentDomain::FileOps);
        assert_eq!(intents[1].domain, IntentDomain::FileOps);
        assert_eq!(intents[2].domain, IntentDomain::FileOps);
        assert_eq!(intents[3].domain, IntentDomain::FileOps);
        assert_eq!(intents[4].domain, IntentDomain::FileOps);
        assert_eq!(intents[5].domain, IntentDomain::FileOps);
        assert_eq!(intents[6].domain, IntentDomain::FileOps);
        assert_eq!(intents[7].domain, IntentDomain::FileOps);

        // DataOps (5 个): grep_pattern, sort_lines, count_pattern, count_file_stats, compare_files
        assert_eq!(intents[8].domain, IntentDomain::DataOps);
        assert_eq!(intents[9].domain, IntentDomain::DataOps);
        assert_eq!(intents[10].domain, IntentDomain::DataOps);
        assert_eq!(intents[11].domain, IntentDomain::DataOps);
        assert_eq!(intents[12].domain, IntentDomain::DataOps);

        // DiagnosticOps (3 个): analyze_errors, check_disk_usage, view_system_logs
        assert_eq!(intents[13].domain, IntentDomain::DiagnosticOps);
        assert_eq!(intents[14].domain, IntentDomain::DiagnosticOps);
        assert_eq!(intents[15].domain, IntentDomain::DiagnosticOps);

        // SystemOps (8 个): list_processes, check_memory_usage, check_cpu_usage,
        //                   check_network_connections, check_uptime, ping_host, view_env_var, check_service_status
        assert_eq!(intents[16].domain, IntentDomain::SystemOps);
        assert_eq!(intents[17].domain, IntentDomain::SystemOps);
        assert_eq!(intents[18].domain, IntentDomain::SystemOps);
        assert_eq!(intents[19].domain, IntentDomain::SystemOps);
        assert_eq!(intents[20].domain, IntentDomain::SystemOps);
        assert_eq!(intents[21].domain, IntentDomain::SystemOps);
        assert_eq!(intents[22].domain, IntentDomain::SystemOps);
        assert_eq!(intents[23].domain, IntentDomain::SystemOps);
    }

    #[test]
    fn test_create_matcher() {
        let builtin = BuiltinIntents::new();
        let matcher = builtin.create_matcher();

        assert_eq!(matcher.len(), 24);
        assert!(!matcher.is_empty());
    }

    #[test]
    fn test_create_engine() {
        let builtin = BuiltinIntents::new();
        let engine = builtin.create_engine();

        assert_eq!(engine.len(), 24);
        assert!(!engine.is_empty());
    }

    #[test]
    fn test_match_count_python_lines() {
        let builtin = BuiltinIntents::new();
        let matcher = builtin.create_matcher();

        let matches = matcher.match_intent("统计 Python 代码行数");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].intent.name, "count_python_lines");
    }

    #[test]
    fn test_match_find_files_by_size() {
        let builtin = BuiltinIntents::new();
        let matcher = builtin.create_matcher();

        let matches = matcher.match_intent("查找大于 100MB 的大文件");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].intent.name, "find_files_by_size");
    }

    #[test]
    fn test_match_grep_pattern() {
        let builtin = BuiltinIntents::new();
        let matcher = builtin.create_matcher();

        let matches = matcher.match_intent("搜索错误模式");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].intent.name, "grep_pattern");
    }

    #[test]
    fn test_template_generation_count_files() {
        let builtin = BuiltinIntents::new();
        let engine = builtin.create_engine();

        let mut bindings = HashMap::new();
        bindings.insert("path".to_string(), ".".to_string());
        bindings.insert("ext".to_string(), "rs".to_string());

        let plan = engine.generate("count_files", bindings).unwrap();
        assert_eq!(plan.command, "find . -name '*.rs' -type f | wc -l");
    }

    #[test]
    fn test_template_generation_grep_pattern() {
        let builtin = BuiltinIntents::new();
        let engine = builtin.create_engine();

        let mut bindings = HashMap::new();
        bindings.insert("pattern".to_string(), "TODO".to_string());
        bindings.insert("path".to_string(), "./src".to_string());

        let plan = engine.generate("grep_pattern", bindings).unwrap();
        assert_eq!(plan.command, "grep -r 'TODO' ./src");
    }

    #[test]
    fn test_template_generation_check_disk_usage() {
        let builtin = BuiltinIntents::new();
        let engine = builtin.create_engine();

        let mut bindings = HashMap::new();
        bindings.insert("path".to_string(), "/var/log".to_string());
        bindings.insert("limit".to_string(), "10".to_string());

        let plan = engine.generate("check_disk_usage", bindings).unwrap();
        assert_eq!(
            plan.command,
            "du -sh /var/log/* | sort -hr | head -n 10"
        );
    }

    #[test]
    fn test_all_templates_have_descriptions() {
        let builtin = BuiltinIntents::new();
        let templates = builtin.all_templates();

        for template in templates {
            assert!(
                !template.description.is_empty(),
                "Template {} missing description",
                template.name
            );
        }
    }

    // ========== Phase 6.2.1: 参数化模板集成测试 ==========

    #[test]
    fn test_find_largest_files_integration() {
        use crate::dsl::intent::extractor::EntityExtractor;

        let builtin = BuiltinIntents::new();
        let matcher = builtin.create_matcher();
        let engine = builtin.create_engine();
        let extractor = EntityExtractor::new();

        // 步骤1: 匹配Intent
        let user_input = "显示当前目录下体积最大的rs文件";
        let matches = matcher.match_intent(user_input);

        assert!(!matches.is_empty(), "应该匹配到Intent");
        let best_match = &matches[0];
        assert_eq!(best_match.intent.name, "find_files_by_size");

        // 步骤2: 提取实体
        let entities = extractor.extract(user_input, &best_match.intent.entities);

        // 验证实体提取
        assert!(entities.contains_key("path"));
        assert!(entities.contains_key("ext"));
        assert!(entities.contains_key("sort_order"));

        // 验证 sort_order 为降序（最大）
        if let Some(EntityType::Custom(_, order)) = entities.get("sort_order") {
            assert_eq!(order, "-hr", "最大应该使用降序");
        } else {
            panic!("应该提取到 sort_order 实体");
        }

        // 步骤3: 生成命令
        let mut bindings = HashMap::new();
        for (key, entity) in entities {
            let value = match entity {
                EntityType::Path(p) => p,
                EntityType::FileType(ft) => ft,
                EntityType::Number(n) => n.to_string(),
                EntityType::Custom(_, v) => v,
                _ => continue,
            };
            bindings.insert(key, value);
        }

        // 添加默认值
        if !bindings.contains_key("limit") {
            bindings.insert("limit".to_string(), "1".to_string());
        }

        let plan = engine.generate("find_files_by_size", bindings).unwrap();

        // 验证生成的命令
        assert!(plan.command.contains("sort -k5 -hr"), "应该使用降序排序");
        assert!(plan.command.contains("*.rs"), "应该过滤rs文件");
    }

    #[test]
    fn test_find_smallest_files_integration() {
        use crate::dsl::intent::extractor::EntityExtractor;

        let builtin = BuiltinIntents::new();
        let matcher = builtin.create_matcher();
        let engine = builtin.create_engine();
        let extractor = EntityExtractor::new();

        // 步骤1: 匹配Intent
        let user_input = "显示当前目录下体积最小的rs文件";
        let matches = matcher.match_intent(user_input);

        assert!(!matches.is_empty(), "应该匹配到Intent");
        let best_match = &matches[0];
        assert_eq!(best_match.intent.name, "find_files_by_size");

        // 步骤2: 提取实体
        let entities = extractor.extract(user_input, &best_match.intent.entities);

        // 验证实体提取
        assert!(entities.contains_key("sort_order"), "应该提取到 sort_order");

        // 验证 sort_order 为升序（最小）
        if let Some(EntityType::Custom(_, order)) = entities.get("sort_order") {
            assert_eq!(order, "-h", "最小应该使用升序");
        } else {
            panic!("应该提取到 sort_order 实体");
        }

        // 步骤3: 生成命令
        let mut bindings = HashMap::new();
        for (key, entity) in entities {
            let value = match entity {
                EntityType::Path(p) => p,
                EntityType::FileType(ft) => ft,
                EntityType::Number(n) => n.to_string(),
                EntityType::Custom(_, v) => v,
                _ => continue,
            };
            bindings.insert(key, value);
        }

        // 添加默认值
        if !bindings.contains_key("limit") {
            bindings.insert("limit".to_string(), "1".to_string());
        }

        let plan = engine.generate("find_files_by_size", bindings).unwrap();

        // 验证生成的命令
        assert!(plan.command.contains("sort -k5 -h"), "应该使用升序排序");
        assert!(!plan.command.contains("-hr"), "不应该使用降序");
        assert!(plan.command.contains("*.rs"), "应该过滤rs文件");
    }

    #[test]
    fn test_largest_vs_smallest_only_direction_differs() {
        use crate::dsl::intent::extractor::EntityExtractor;

        let builtin = BuiltinIntents::new();
        let matcher = builtin.create_matcher();
        let engine = builtin.create_engine();
        let extractor = EntityExtractor::new();

        // 最大文件
        let largest_input = "显示当前目录下体积最大的rs文件";
        let largest_matches = matcher.match_intent(largest_input);
        let largest_entities = extractor.extract(largest_input, &largest_matches[0].intent.entities);

        // 最小文件
        let smallest_input = "显示当前目录下体积最小的rs文件";
        let smallest_matches = matcher.match_intent(smallest_input);
        let smallest_entities = extractor.extract(smallest_input, &smallest_matches[0].intent.entities);

        // 验证：Intent相同
        assert_eq!(largest_matches[0].intent.name, smallest_matches[0].intent.name);

        // 验证：sort_order 不同
        let largest_order = if let Some(EntityType::Custom(_, order)) = largest_entities.get("sort_order") {
            order.clone()
        } else {
            panic!("应该提取到 sort_order");
        };

        let smallest_order = if let Some(EntityType::Custom(_, order)) = smallest_entities.get("sort_order") {
            order.clone()
        } else {
            panic!("应该提取到 sort_order");
        };

        assert_eq!(largest_order, "-hr", "最大 = 降序");
        assert_eq!(smallest_order, "-h", "最小 = 升序");
        assert_ne!(largest_order, smallest_order, "排序方向应该不同");

        // 哲学验证：象不变，爻可变
        // - 象（不变）：都是 find_files_by_size Intent
        // - 爻（变化）：只有 sort_order 参数不同
    }
}
