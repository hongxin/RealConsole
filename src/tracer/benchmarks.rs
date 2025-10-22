//! 性能基准测试
//!
//! 测试 UnifiedTracer 的关键操作性能

#[cfg(test)]
mod benches {
    use super::super::*;
    use crate::config::ConversationConfig;
    use crate::conversation::context_manager::ContextManager;
    use crate::execution_logger::{CommandType, ExecutionLogger};
    use crate::history::HistoryManager;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::Instant;
    use tokio::sync::RwLock;

    /// 创建测试用的 UnifiedTracer（带有大量数据）
    fn create_large_tracer(entry_count: usize) -> Arc<UnifiedTracer> {
        let test_path = PathBuf::from("/tmp/realconsole_bench_history.json");
        let mut history = HistoryManager::new(test_path, entry_count * 2);

        // 添加大量历史记录
        for i in 0..entry_count {
            history.add(format!("command_{}", i), i % 2 == 0);
        }

        let mut exec_logger = ExecutionLogger::new(entry_count * 2);
        // 添加大量执行日志
        for i in 0..entry_count {
            exec_logger.log(
                format!("cmd_{}", i),
                CommandType::Shell,
                i % 2 == 0,
                std::time::Duration::from_millis(i as u64 % 100),
                &format!("result_{}", i),
            );
        }

        let config = ConversationConfig::default();
        let context = ContextManager::new(config);

        Arc::new(UnifiedTracer::new(
            Arc::new(RwLock::new(history)),
            Arc::new(RwLock::new(exec_logger)),
            None,
            Arc::new(RwLock::new(context)),
        ))
    }

    #[tokio::test]
    async fn bench_query_all_100_entries() {
        let tracer = create_large_tracer(1000);

        let start = Instant::now();
        let result = tracer.query_all(100).await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        println!("query_all(100): {:?}", duration);
        assert!(duration.as_millis() < 50, "查询100条应该 < 50ms");
    }

    #[tokio::test]
    async fn bench_query_by_dimension() {
        let tracer = create_large_tracer(1000);

        let start = Instant::now();
        let result = tracer
            .query_by_dimension(Dimension::Statistics, 100)
            .await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        println!("query_by_dimension(100): {:?}", duration);
        assert!(
            duration.as_millis() < 30,
            "按维度查询100条应该 < 30ms"
        );
    }

    #[tokio::test]
    async fn bench_search_keyword() {
        let tracer = create_large_tracer(1000);

        let start = Instant::now();
        let result = tracer.search("command").await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        println!("search(\"command\"): {:?}", duration);
        assert!(duration.as_millis() < 100, "搜索应该 < 100ms");
    }

    #[tokio::test]
    async fn bench_deduplicate() {
        let tracer = create_large_tracer(500);

        // 创建包含重复的条目列表
        let mut entries = Vec::new();
        for _ in 0..3 {
            let batch = tracer.query_all(100).await.unwrap();
            entries.extend(batch);
        }

        let start = Instant::now();
        let deduplicated = UnifiedTracer::deduplicate_entries(entries);
        let duration = start.elapsed();

        println!("deduplicate(300 entries): {:?}", duration);
        println!("去重后: {} 条", deduplicated.len());
        assert!(duration.as_millis() < 10, "去重300条应该 < 10ms");
    }

    #[tokio::test]
    async fn bench_stats() {
        let tracer = create_large_tracer(1000);

        let start = Instant::now();
        let result = tracer.stats().await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        println!("stats(): {:?}", duration);
        assert!(duration.as_millis() < 200, "统计分析应该 < 200ms");
    }

    #[tokio::test]
    async fn bench_parallel_queries() {
        let tracer = create_large_tracer(1000);

        let start = Instant::now();

        // 并行执行多个查询
        let (r1, r2, r3, r4) = tokio::join!(
            tracer.query_by_dimension(Dimension::Statistics, 50),
            tracer.query_by_dimension(Dimension::Coordination, 50),
            tracer.search("command"),
            tracer.stats()
        );

        let duration = start.elapsed();

        assert!(r1.is_ok() && r2.is_ok() && r3.is_ok() && r4.is_ok());
        println!("4 parallel queries: {:?}", duration);
        assert!(duration.as_millis() < 250, "4个并行查询应该 < 250ms");
    }
}
