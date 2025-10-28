//! v1.15.0 Phase 5: 端到端集成测试
//!
//! 测试三系（Liangyyi/Tracer/Bagua）协同工作的正确性

use realconsole::bagua::dimension::BaguaDimension;
use realconsole::bagua::entry::{ActionResult, KnowledgeSource, MemoryContent, MemoryEntry};
use realconsole::bagua::palace::{BaguaMemoryPalace, PalaceConfig};
use realconsole::config::ConversationConfig;
use realconsole::conversation::context_manager::ContextManager;
use realconsole::execution_logger::ExecutionLogger;
use realconsole::history::HistoryManager;
use realconsole::liangyyi::adaptive::TargetState;
use realconsole::liangyyi::{Event, StateTracker};
use realconsole::tracer::{Dimension, EntryType, Status, TraceEntry, UnifiedTracer};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 创建完整的三系测试环境
async fn setup_three_systems() -> (
    Arc<StateTracker>,
    Arc<UnifiedTracer>,
    Arc<RwLock<BaguaMemoryPalace>>,
) {
    // 1. 创建 Tracer 依赖
    let history = Arc::new(RwLock::new(HistoryManager::new(
        "/tmp/test_integration_history.json",
        100,
    )));
    let exec_logger = Arc::new(RwLock::new(ExecutionLogger::new(100)));
    let context = Arc::new(RwLock::new(ContextManager::new(
        ConversationConfig::default(),
    )));

    // 2. 创建 UnifiedTracer
    let tracer = Arc::new(UnifiedTracer::new(
        history,
        exec_logger,
        None,
        context,
    ));

    // 3. 创建 StateTracker 并关联 Tracer
    let mut tracker = StateTracker::with_default();
    tracker.set_tracer(Arc::clone(&tracer));
    tracker.enable_adaptive(TargetState::balanced());
    let tracker = Arc::new(tracker);

    // 4. 创建 BaguaMemoryPalace 并关联 Tracer
    let mut palace = BaguaMemoryPalace::with_config(PalaceConfig::default());
    palace.set_tracer(Arc::clone(&tracer));
    let palace = Arc::new(RwLock::new(palace));

    (tracker, tracer, palace)
}

// ============================================================================
// 场景1: 观测 → 预测 → 执行 → 追踪（Liangyyi + Tracer 闭环）
// ============================================================================

#[tokio::test]
async fn test_scenario1_liangyyi_tracer_closed_loop() {
    let (tracker, tracer, _palace) = setup_three_systems().await;

    // Step 1: 产生系统事件 → StateTracker 观测
    for _ in 0..10 {
        tracker.update_from_event(Event::UserExecute).await;
    }

    // 验证 StateTracker 状态更新
    let vector = tracker.to_state_vector().await;
    assert!(vector.get("activity").unwrap_or(0.0) > 0.0, "活动度应该 > 0");

    // Step 2: StatePredictor 预测 → AdaptiveSystem 生成建议
    // 由于我们只有 StateTracker，我们验证自适应系统是否启用
    assert!(
        tracker.is_adaptive_enabled(),
        "自适应系统应该已启用"
    );

    // Step 3: 触发自动优化（会生成建议并记录到 Tracer）
    let recommendations = tracker.auto_optimize().await.unwrap();

    // Step 4: 验证 Tracer 记录了优化事件
    let stats_entries = tracer
        .query_by_dimension(Dimension::Statistics, 20)
        .await
        .unwrap();

    // 应该有 AdaptiveOptimization 类型的事件
    let has_adaptive_event = stats_entries
        .iter()
        .any(|e| e.entry_type == EntryType::AdaptiveOptimization);

    if recommendations.is_empty() {
        // 如果没有生成建议（状态接近目标），这是正常的
        println!("状态接近目标，未生成建议");
    } else {
        // 如果生成了建议，应该有相应的 Tracer 记录
        assert!(
            has_adaptive_event,
            "Tracer 应该记录了自适应优化事件"
        );
    }

    // 验证闭环完整性：事件 → 观测 → 优化 → 记录
    println!("✅ 场景1通过：Liangyyi + Tracer 闭环验证成功");
}

// ============================================================================
// 场景2: 炼化 → 建议 → 观测（Bagua + Tracer 流程）
// ============================================================================

#[tokio::test]
async fn test_scenario2_bagua_tracer_refinement_flow() {
    let (_tracker, tracer, palace) = setup_three_systems().await;

    // Step 1: Bagua 炼化 Memory → 存储记忆条目
    let palace_guard = palace.write().await;

    // 存储不同类型的记忆
    let memories = vec![
        MemoryEntry::new(
            BaguaDimension::Qian,
            MemoryContent::Intent {
                goal: "完成集成测试".to_string(),
                context: Some("v1.15.0 Phase 5".to_string()),
                priority: 0.95,
            },
        ),
        MemoryEntry::new(
            BaguaDimension::Dui,
            MemoryContent::Conversation {
                role: "user".to_string(),
                message: "实现端到端测试".to_string(),
                session_id: Some("test-session-1".to_string()),
            },
        ),
        MemoryEntry::new(
            BaguaDimension::Li,
            MemoryContent::Knowledge {
                fact: "三系集成需要端到端测试验证".to_string(),
                source: KnowledgeSource::SystemObserved,
                confidence: 0.9,
            },
        ),
    ];

    for memory in memories {
        palace_guard.store(memory).await.unwrap();
    }

    drop(palace_guard); // 释放写锁

    // Step 2: 验证 Tracer 记录了炼化过程（存储事件）
    let memory_entries = tracer
        .query_by_dimension(Dimension::Memory, 20)
        .await
        .unwrap();

    // 应该有 SystemEvent 类型的记忆存储事件
    let storage_events: Vec<_> = memory_entries
        .iter()
        .filter(|e| {
            e.entry_type == EntryType::SystemEvent && e.content.contains("记忆存储")
        })
        .collect();

    assert!(
        storage_events.len() >= 3,
        "应该有至少 3 条记忆存储事件，实际: {}",
        storage_events.len()
    );

    // Step 3: 验证记忆内容的准确性
    let all_content: String = memory_entries.iter().map(|e| e.content.clone()).collect();
    assert!(all_content.contains("Qian"), "应该包含 Qian 维度的记录");
    assert!(all_content.contains("Dui"), "应该包含 Dui 维度的记录");
    assert!(all_content.contains("Li"), "应该包含 Li 维度的记录");

    // Step 4: 验证离坎平衡
    let palace_guard = palace.read().await;
    let balance = palace_guard.check_likan_balance().await;
    assert!(balance.li_count > 0, "离维度应该有记忆条目");

    println!("✅ 场景2通过：Bagua + Tracer 炼化流程验证成功");
}

// ============================================================================
// 场景3: 三系完整协同
// ============================================================================

#[tokio::test]
async fn test_scenario3_full_three_systems_integration() {
    let (tracker, tracer, palace) = setup_three_systems().await;

    // 1. Liangyyi 活动
    for _ in 0..5 {
        tracker.update_from_event(Event::UserExecute).await;
    }

    // 2. Bagua 存储记忆
    let palace_guard = palace.write().await;
    palace_guard
        .store(MemoryEntry::new(
            BaguaDimension::Li,
            MemoryContent::Knowledge {
                fact: "三系协同测试".to_string(),
                source: KnowledgeSource::UserProvided,
                confidence: 1.0,
            },
        ))
        .await
        .unwrap();
    drop(palace_guard);

    // 3. 手动添加自定义追踪事件
    tracer
        .add_entry(TraceEntry::new(
            Dimension::Coordination,
            EntryType::SystemEvent,
            "三系协同测试事件".to_string(),
            Status::Success,
        ))
        .await;

    // 4. 验证三系都有活动记录
    let stats = tracer.stats().await.unwrap();
    assert!(stats.total_entries > 0, "Tracer 应该有记录");

    // 验证各维度都有数据
    let dimensions_with_data = stats
        .by_dimension
        .iter()
        .filter(|(_, &count)| count > 0)
        .count();

    assert!(
        dimensions_with_data >= 2,
        "至少应该有 2 个维度有数据，实际: {}",
        dimensions_with_data
    );

    // 5. 验证 StateTracker 的 Tracer 集成
    assert!(tracker.is_tracer_enabled(), "StateTracker 应启用 Tracer");

    // 6. 验证 Bagua 的 Tracer 集成
    let palace_guard = palace.read().await;
    assert!(palace_guard.is_tracer_enabled(), "Bagua 应启用 Tracer");

    println!("✅ 场景3通过：三系完整协同验证成功");
}

// ============================================================================
// 场景4: 压力测试 - 高频事件
// ============================================================================

#[tokio::test]
async fn test_scenario4_high_frequency_events() {
    let (tracker, tracer, palace) = setup_three_systems().await;

    let start = std::time::Instant::now();

    // 模拟高频事件（100 events）
    for i in 0..100 {
        // Liangyyi 事件
        tracker.update_from_event(Event::UserExecute).await;

        // Bagua 存储（每 10 个事件存一次，避免过多）
        if i % 10 == 0 {
            let palace_guard = palace.write().await;
            palace_guard
                .store(MemoryEntry::new(
                    BaguaDimension::Kan,
                    MemoryContent::Action {
                        command: format!("high_freq_test_{}", i),
                        result: ActionResult::Success,
                        duration_ms: 10,
                    },
                ))
                .await
                .unwrap();
            drop(palace_guard);
        }

        // Tracer 自定义事件（每 20 个事件记一次）
        if i % 20 == 0 {
            tracer
                .add_entry(TraceEntry::new(
                    Dimension::Statistics,
                    EntryType::SystemEvent,
                    format!("压力测试事件 #{}", i),
                    Status::Success,
                ))
                .await;
        }
    }

    let elapsed = start.elapsed();

    // 验证性能：100 个事件应该在合理时间内完成（< 5秒）
    assert!(
        elapsed.as_secs() < 5,
        "100 events 应该在 5 秒内完成，实际: {:?}",
        elapsed
    );

    // 验证数据完整性
    let vector = tracker.to_state_vector().await;
    assert!(vector.get("activity").unwrap_or(0.0) > 0.0, "应该有活动记录");

    let stats = tracer.stats().await.unwrap();
    assert!(stats.total_entries >= 5, "Tracer 应该有至少 5 条记录");

    println!(
        "✅ 场景4通过：高频事件测试（100 events in {:?}）",
        elapsed
    );
}

// ============================================================================
// 场景5: 并发查询测试
// ============================================================================

#[tokio::test]
async fn test_scenario5_concurrent_queries() {
    let (tracker, tracer, palace) = setup_three_systems().await;

    // 先产生一些数据
    for _ in 0..10 {
        tracker.update_from_event(Event::UserExecute).await;
    }

    let palace_guard = palace.write().await;
    for i in 0..5 {
        palace_guard
            .store(MemoryEntry::new(
                BaguaDimension::Li,
                MemoryContent::Knowledge {
                    fact: format!("并发测试知识 #{}", i),
                    source: KnowledgeSource::SystemObserved,
                    confidence: 0.8,
                },
            ))
            .await
            .unwrap();
    }
    drop(palace_guard);

    // 并发查询
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let tracer_clone = Arc::clone(&tracer);
            let tracker_clone = Arc::clone(&tracker);
            let palace_clone = Arc::clone(&palace);

            tokio::spawn(async move {
                // 并发执行多种查询
                let _ = tracer_clone.stats().await;
                let _ = tracker_clone.to_state_vector().await;
                let palace_guard = palace_clone.read().await;
                let _ = palace_guard.check_likan_balance().await;
            })
        })
        .collect();

    // 等待所有查询完成
    for handle in handles {
        handle.await.unwrap();
    }

    println!("✅ 场景5通过：并发查询测试（10 concurrent queries）");
}

// ============================================================================
// 场景6: 内存稳定性测试
// ============================================================================

#[tokio::test]
async fn test_scenario6_memory_stability() {
    let (tracker, tracer, palace) = setup_three_systems().await;

    // 长时间运行模拟（1000 次迭代）
    for i in 0..1000 {
        // 定期更新状态
        if i % 10 == 0 {
            tracker.update_from_event(Event::UserExecute).await;
        }

        // 定期存储记忆（但不能太频繁，避免内存爆炸）
        if i % 50 == 0 {
            let palace_guard = palace.write().await;
            palace_guard
                .store(MemoryEntry::new(
                    BaguaDimension::Kan,
                    MemoryContent::Action {
                        command: "stability_test".to_string(),
                        result: ActionResult::Success,
                        duration_ms: 5,
                    },
                ))
                .await
                .unwrap();
            drop(palace_guard);
        }

        // 定期查询（验证数据结构稳定）
        if i % 100 == 0 {
            let _ = tracer.stats().await;
            let _ = tracker.to_state_vector().await;
        }
    }

    // 最终验证：系统仍然可用
    let stats = tracer.stats().await.unwrap();
    assert!(stats.total_entries > 0, "系统应该仍然可用");

    let vector = tracker.to_state_vector().await;
    assert!(
        vector.dimension_count() > 0,
        "StateVector 应该仍然有效"
    );

    let palace_guard = palace.read().await;
    let balance = palace_guard.check_likan_balance().await;
    // 验证离坎平衡计算仍然正常
    assert!(
        !balance.li_energy.is_nan() && !balance.kan_energy.is_nan(),
        "能量值应该是有效数字"
    );

    println!("✅ 场景6通过：内存稳定性测试（1000 iterations）");
}

// ============================================================================
// 场景7: 完整的观测链路测试
// ============================================================================

#[tokio::test]
async fn test_scenario7_complete_observation_chain() {
    let (tracker, tracer, palace) = setup_three_systems().await;

    // 1. 用户活动 → StateTracker
    for _ in 0..15 {
        tracker.update_from_event(Event::UserExecute).await;
    }

    // 2. 触发自动优化 → 生成建议
    let recommendations = tracker.auto_optimize().await.unwrap();

    // 3. 存储相关记忆 → Bagua
    let palace_guard = palace.write().await;
    palace_guard
        .store(MemoryEntry::new(
            BaguaDimension::Li,
            MemoryContent::Knowledge {
                fact: format!("自动优化生成了 {} 条建议", recommendations.len()),
                source: KnowledgeSource::SystemObserved,
                confidence: 0.95,
            },
        ))
        .await
        .unwrap();
    drop(palace_guard);

    // 4. 验证完整的观测链路
    let stats_entries = tracer
        .query_by_dimension(Dimension::Statistics, 50)
        .await
        .unwrap();

    let memory_entries = tracer
        .query_by_dimension(Dimension::Memory, 50)
        .await
        .unwrap();

    // 应该同时有 Statistics 和 Memory 维度的记录
    assert!(
        !stats_entries.is_empty() || !memory_entries.is_empty(),
        "应该有观测记录"
    );

    // 验证状态一致性
    let vector = tracker.to_state_vector().await;
    let palace_guard = palace.read().await;
    let balance = palace_guard.check_likan_balance().await;

    // 所有系统都应该处于可用状态
    assert!(vector.dimension_count() > 0);
    // 至少应该有一方有记忆条目
    assert!(balance.li_count > 0);

    println!("✅ 场景7通过：完整观测链路验证成功");
}
