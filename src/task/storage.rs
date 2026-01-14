//! Task Storage 适配器
//!
//! v1.109.0: 将 Task 系统存储迁移到 Storage Layer 2.0
//!
//! 提供任务执行计划和结果的持久化存储

use super::types::{ExecutionPlan, ExecutionResult, TaskResult, TaskStatus};
use crate::storage::{StorageBackend, StorageError, StorageResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Task 存储适配器
///
/// 将 Task 系统与 StorageBackend 集成
pub struct TaskStorageAdapter<S: StorageBackend> {
    /// 存储后端
    storage: Arc<S>,
    /// 存储键前缀
    prefix: String,
    /// 配置
    config: TaskStorageConfig,
}

/// Task 存储配置
#[derive(Debug, Clone)]
pub struct TaskStorageConfig {
    /// 最大历史记录数
    pub max_history: usize,
    /// 是否存储详细结果
    pub store_detailed_results: bool,
    /// 结果过期时间（天，0=永不过期）
    pub result_expiry_days: u32,
}

impl Default for TaskStorageConfig {
    fn default() -> Self {
        Self {
            max_history: 100,
            store_detailed_results: true,
            result_expiry_days: 30,
        }
    }
}

/// 存储的执行计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPlan {
    /// 计划数据
    pub plan: ExecutionPlan,
    /// 存储时间
    pub stored_at: DateTime<Utc>,
    /// 状态
    pub status: PlanStatus,
    /// 执行结果 ID（如果已执行）
    pub result_id: Option<String>,
}

/// 计划状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlanStatus {
    /// 待执行
    Pending,
    /// 执行中
    Running,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 存储的执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredResult {
    /// 结果数据
    pub result: ExecutionResult,
    /// 存储时间
    pub stored_at: DateTime<Utc>,
    /// 计划 ID
    pub plan_id: String,
    /// 计划目标
    pub goal: String,
}

/// Task 历史索引
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskIndex {
    /// 版本号
    pub version: u32,
    /// 计划列表
    pub plans: Vec<TaskIndexEntry>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

/// 索引条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskIndexEntry {
    /// 计划 ID
    pub plan_id: String,
    /// 目标
    pub goal: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 状态
    pub status: PlanStatus,
    /// 任务数
    pub task_count: usize,
    /// 成功率（如果已完成）
    pub success_rate: Option<f64>,
}

impl<S: StorageBackend> TaskStorageAdapter<S> {
    /// 创建新的适配器
    pub fn new(storage: Arc<S>) -> Self {
        Self::with_config(storage, TaskStorageConfig::default())
    }

    /// 使用配置创建
    pub fn with_config(storage: Arc<S>, config: TaskStorageConfig) -> Self {
        Self {
            storage,
            prefix: "tasks".to_string(),
            config,
        }
    }

    /// 设置前缀
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// 获取计划存储键
    fn plan_key(&self, id: &str) -> String {
        format!("{}/plans/{}", self.prefix, id)
    }

    /// 获取结果存储键
    fn result_key(&self, id: &str) -> String {
        format!("{}/results/{}", self.prefix, id)
    }

    /// 获取索引键
    fn index_key(&self) -> String {
        format!("{}/index", self.prefix)
    }

    /// 保存执行计划
    pub async fn save_plan(&self, plan: &ExecutionPlan) -> StorageResult<()> {
        let stored = StoredPlan {
            plan: plan.clone(),
            stored_at: Utc::now(),
            status: PlanStatus::Pending,
            result_id: None,
        };

        let data = serde_json::to_vec(&stored).map_err(|e| {
            StorageError::Serialization(format!("Failed to serialize plan: {}", e))
        })?;

        self.storage.write(&self.plan_key(&plan.id), &data).await?;

        // 更新索引
        self.update_index_for_plan(plan, PlanStatus::Pending, None)
            .await?;

        Ok(())
    }

    /// 加载执行计划
    pub async fn load_plan(&self, id: &str) -> StorageResult<StoredPlan> {
        let data = self.storage.read(&self.plan_key(id)).await?;

        serde_json::from_slice(&data).map_err(|e| {
            StorageError::Serialization(format!("Failed to deserialize plan: {}", e))
        })
    }

    /// 更新计划状态
    pub async fn update_plan_status(
        &self,
        id: &str,
        status: PlanStatus,
        result_id: Option<String>,
    ) -> StorageResult<()> {
        let mut stored = self.load_plan(id).await?;
        stored.status = status.clone();
        stored.result_id = result_id.clone();

        let data = serde_json::to_vec(&stored).map_err(|e| {
            StorageError::Serialization(format!("Failed to serialize plan: {}", e))
        })?;

        self.storage.write(&self.plan_key(id), &data).await?;

        // 更新索引
        let success_rate = if let Some(ref rid) = result_id {
            if let Ok(result) = self.load_result(rid).await {
                Some(result.result.success_rate())
            } else {
                None
            }
        } else {
            None
        };

        self.update_index_status(id, status, success_rate).await?;

        Ok(())
    }

    /// 删除计划
    pub async fn delete_plan(&self, id: &str) -> StorageResult<()> {
        // 先尝试加载以获取关联的结果
        if let Ok(stored) = self.load_plan(id).await {
            if let Some(result_id) = stored.result_id {
                let _ = self.storage.delete(&self.result_key(&result_id)).await;
            }
        }

        self.storage.delete(&self.plan_key(id)).await?;
        self.remove_from_index(id).await?;

        Ok(())
    }

    /// 保存执行结果
    pub async fn save_result(
        &self,
        result: &ExecutionResult,
        goal: &str,
    ) -> StorageResult<String> {
        let result_id = format!("result-{}", uuid::Uuid::new_v4());

        let stored = StoredResult {
            result: result.clone(),
            stored_at: Utc::now(),
            plan_id: result.plan_id.clone(),
            goal: goal.to_string(),
        };

        let data = serde_json::to_vec(&stored).map_err(|e| {
            StorageError::Serialization(format!("Failed to serialize result: {}", e))
        })?;

        self.storage.write(&self.result_key(&result_id), &data).await?;

        // 更新计划状态
        let status = if result.is_success() {
            PlanStatus::Completed
        } else {
            PlanStatus::Failed
        };

        self.update_plan_status(&result.plan_id, status, Some(result_id.clone()))
            .await?;

        Ok(result_id)
    }

    /// 加载执行结果
    pub async fn load_result(&self, id: &str) -> StorageResult<StoredResult> {
        let data = self.storage.read(&self.result_key(id)).await?;

        serde_json::from_slice(&data).map_err(|e| {
            StorageError::Serialization(format!("Failed to deserialize result: {}", e))
        })
    }

    /// 获取计划的执行结果
    pub async fn get_plan_result(&self, plan_id: &str) -> StorageResult<Option<StoredResult>> {
        let stored = self.load_plan(plan_id).await?;

        if let Some(result_id) = stored.result_id {
            Ok(Some(self.load_result(&result_id).await?))
        } else {
            Ok(None)
        }
    }

    /// 加载索引
    async fn load_index(&self) -> StorageResult<TaskIndex> {
        match self.storage.read(&self.index_key()).await {
            Ok(data) => serde_json::from_slice(&data).map_err(|e| {
                StorageError::Serialization(format!("Failed to deserialize index: {}", e))
            }),
            Err(StorageError::NotFound(_)) => Ok(TaskIndex {
                version: 1,
                plans: Vec::new(),
                updated_at: Utc::now(),
            }),
            Err(e) => Err(e),
        }
    }

    /// 保存索引
    async fn save_index(&self, index: &TaskIndex) -> StorageResult<()> {
        let data = serde_json::to_vec(index).map_err(|e| {
            StorageError::Serialization(format!("Failed to serialize index: {}", e))
        })?;

        self.storage.write(&self.index_key(), &data).await
    }

    /// 更新索引（添加计划）
    async fn update_index_for_plan(
        &self,
        plan: &ExecutionPlan,
        status: PlanStatus,
        success_rate: Option<f64>,
    ) -> StorageResult<()> {
        let mut index = self.load_index().await?;

        // 移除已存在的
        index.plans.retain(|e| e.plan_id != plan.id);

        // 添加新条目
        index.plans.push(TaskIndexEntry {
            plan_id: plan.id.clone(),
            goal: plan.goal.clone(),
            created_at: plan.created_at,
            status,
            task_count: plan.total_tasks(),
            success_rate,
        });

        // 按创建时间倒序
        index.plans.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // 限制历史数量
        if index.plans.len() > self.config.max_history {
            index.plans.truncate(self.config.max_history);
        }

        index.updated_at = Utc::now();
        self.save_index(&index).await
    }

    /// 更新索引状态
    async fn update_index_status(
        &self,
        plan_id: &str,
        status: PlanStatus,
        success_rate: Option<f64>,
    ) -> StorageResult<()> {
        let mut index = self.load_index().await?;

        if let Some(entry) = index.plans.iter_mut().find(|e| e.plan_id == plan_id) {
            entry.status = status;
            entry.success_rate = success_rate;
        }

        index.updated_at = Utc::now();
        self.save_index(&index).await
    }

    /// 从索引中移除
    async fn remove_from_index(&self, plan_id: &str) -> StorageResult<()> {
        let mut index = self.load_index().await?;
        index.plans.retain(|e| e.plan_id != plan_id);
        index.updated_at = Utc::now();
        self.save_index(&index).await
    }

    /// 列出所有计划
    pub async fn list_plans(&self) -> StorageResult<Vec<TaskIndexEntry>> {
        let index = self.load_index().await?;
        Ok(index.plans)
    }

    /// 按状态过滤计划
    pub async fn list_by_status(&self, status: PlanStatus) -> StorageResult<Vec<TaskIndexEntry>> {
        let index = self.load_index().await?;
        Ok(index
            .plans
            .into_iter()
            .filter(|e| e.status == status)
            .collect())
    }

    /// 搜索计划
    pub async fn search(&self, query: &str) -> StorageResult<Vec<TaskIndexEntry>> {
        let index = self.load_index().await?;
        let query_lower = query.to_lowercase();

        Ok(index
            .plans
            .into_iter()
            .filter(|e| {
                e.goal.to_lowercase().contains(&query_lower)
                    || e.plan_id.contains(query)
            })
            .collect())
    }

    /// 获取统计信息
    pub async fn stats(&self) -> StorageResult<TaskStorageStats> {
        let index = self.load_index().await?;

        let total_plans = index.plans.len();
        let completed = index
            .plans
            .iter()
            .filter(|e| e.status == PlanStatus::Completed)
            .count();
        let failed = index
            .plans
            .iter()
            .filter(|e| e.status == PlanStatus::Failed)
            .count();
        let pending = index
            .plans
            .iter()
            .filter(|e| e.status == PlanStatus::Pending)
            .count();

        let total_tasks: usize = index.plans.iter().map(|e| e.task_count).sum();

        let avg_success_rate = {
            let rates: Vec<f64> = index
                .plans
                .iter()
                .filter_map(|e| e.success_rate)
                .collect();
            if rates.is_empty() {
                None
            } else {
                Some(rates.iter().sum::<f64>() / rates.len() as f64)
            }
        };

        Ok(TaskStorageStats {
            total_plans,
            completed_plans: completed,
            failed_plans: failed,
            pending_plans: pending,
            total_tasks,
            average_success_rate: avg_success_rate,
            index_updated_at: index.updated_at,
        })
    }

    /// 清理过期结果
    pub async fn cleanup_expired(&self) -> StorageResult<usize> {
        if self.config.result_expiry_days == 0 {
            return Ok(0);
        }

        let expiry = Utc::now() - chrono::Duration::days(self.config.result_expiry_days as i64);
        let index = self.load_index().await?;

        let mut removed = 0;
        for entry in index.plans.iter() {
            if entry.created_at < expiry
                && entry.status != PlanStatus::Running
                && self.delete_plan(&entry.plan_id).await.is_ok()
            {
                removed += 1;
            }
        }

        Ok(removed)
    }

    /// 清空所有数据
    pub async fn clear(&self) -> StorageResult<usize> {
        let keys = self.storage.list(&self.prefix).await?;
        let count = keys.len();

        for key in keys {
            self.storage.delete(&key).await?;
        }

        Ok(count)
    }
}

/// Task 存储统计
#[derive(Debug, Clone)]
pub struct TaskStorageStats {
    /// 总计划数
    pub total_plans: usize,
    /// 已完成计划数
    pub completed_plans: usize,
    /// 失败计划数
    pub failed_plans: usize,
    /// 待执行计划数
    pub pending_plans: usize,
    /// 总任务数
    pub total_tasks: usize,
    /// 平均成功率
    pub average_success_rate: Option<f64>,
    /// 索引更新时间
    pub index_updated_at: DateTime<Utc>,
}

/// Task 存储后端包装
pub struct TaskAsStorage<S: StorageBackend> {
    adapter: TaskStorageAdapter<S>,
}

impl<S: StorageBackend> TaskAsStorage<S> {
    pub fn new(adapter: TaskStorageAdapter<S>) -> Self {
        Self { adapter }
    }
}

#[async_trait]
impl<S: StorageBackend + 'static> StorageBackend for TaskAsStorage<S> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.adapter.storage.read(key).await
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        self.adapter.storage.write(key, data).await
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        self.adapter.storage.delete(key).await
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        self.adapter.storage.list(prefix).await
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        self.adapter.storage.exists(key).await
    }

    fn stats(&self) -> crate::storage::StorageStats {
        self.adapter.storage.stats()
    }

    fn name(&self) -> &'static str {
        "TaskAsStorage"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;
    use crate::task::types::{ExecutionMode, ExecutionStage, SubTask};

    fn create_test_adapter() -> TaskStorageAdapter<MemoryStorage> {
        let storage = Arc::new(MemoryStorage::new());
        TaskStorageAdapter::new(storage)
    }

    fn create_test_plan(id: &str, goal: &str) -> ExecutionPlan {
        let task = SubTask::new("task-1", "Test Task", "echo test");
        let stage = ExecutionStage::new(1, vec![task], ExecutionMode::Sequential);
        let mut plan = ExecutionPlan::new(goal, vec![stage]);
        plan.id = id.to_string();
        plan
    }

    fn create_test_result(plan_id: &str) -> ExecutionResult {
        ExecutionResult {
            plan_id: plan_id.to_string(),
            total_tasks: 2,
            completed_tasks: 2,
            failed_tasks: 0,
            skipped_tasks: 0,
            total_time: 10,
            task_results: vec![],
        }
    }

    #[tokio::test]
    async fn test_adapter_new() {
        let adapter = create_test_adapter();
        let plans = adapter.list_plans().await.unwrap();
        assert!(plans.is_empty());
    }

    #[tokio::test]
    async fn test_save_and_load_plan() {
        let adapter = create_test_adapter();

        let plan = create_test_plan("plan-1", "Test Goal");
        adapter.save_plan(&plan).await.unwrap();

        let loaded = adapter.load_plan("plan-1").await.unwrap();
        assert_eq!(loaded.plan.id, "plan-1");
        assert_eq!(loaded.plan.goal, "Test Goal");
        assert_eq!(loaded.status, PlanStatus::Pending);
    }

    #[tokio::test]
    async fn test_update_plan_status() {
        let adapter = create_test_adapter();

        let plan = create_test_plan("plan-status", "Status Test");
        adapter.save_plan(&plan).await.unwrap();

        adapter
            .update_plan_status("plan-status", PlanStatus::Running, None)
            .await
            .unwrap();

        let loaded = adapter.load_plan("plan-status").await.unwrap();
        assert_eq!(loaded.status, PlanStatus::Running);
    }

    #[tokio::test]
    async fn test_delete_plan() {
        let adapter = create_test_adapter();

        let plan = create_test_plan("plan-del", "Delete Test");
        adapter.save_plan(&plan).await.unwrap();

        adapter.delete_plan("plan-del").await.unwrap();

        let result = adapter.load_plan("plan-del").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_save_and_load_result() {
        let adapter = create_test_adapter();

        let plan = create_test_plan("plan-result", "Result Test");
        adapter.save_plan(&plan).await.unwrap();

        let result = create_test_result("plan-result");
        let result_id = adapter.save_result(&result, "Result Test").await.unwrap();

        let loaded = adapter.load_result(&result_id).await.unwrap();
        assert_eq!(loaded.result.plan_id, "plan-result");
        assert_eq!(loaded.result.total_tasks, 2);
    }

    #[tokio::test]
    async fn test_get_plan_result() {
        let adapter = create_test_adapter();

        let plan = create_test_plan("plan-get-result", "Get Result");
        adapter.save_plan(&plan).await.unwrap();

        // 没有结果时
        let result = adapter.get_plan_result("plan-get-result").await.unwrap();
        assert!(result.is_none());

        // 保存结果后
        let exec_result = create_test_result("plan-get-result");
        adapter.save_result(&exec_result, "Get Result").await.unwrap();

        let result = adapter.get_plan_result("plan-get-result").await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_list_plans() {
        let adapter = create_test_adapter();

        adapter
            .save_plan(&create_test_plan("p1", "Goal 1"))
            .await
            .unwrap();
        adapter
            .save_plan(&create_test_plan("p2", "Goal 2"))
            .await
            .unwrap();
        adapter
            .save_plan(&create_test_plan("p3", "Goal 3"))
            .await
            .unwrap();

        let plans = adapter.list_plans().await.unwrap();
        assert_eq!(plans.len(), 3);
    }

    #[tokio::test]
    async fn test_list_by_status() {
        let adapter = create_test_adapter();

        adapter
            .save_plan(&create_test_plan("s1", "Goal 1"))
            .await
            .unwrap();
        adapter
            .save_plan(&create_test_plan("s2", "Goal 2"))
            .await
            .unwrap();

        adapter
            .update_plan_status("s1", PlanStatus::Completed, None)
            .await
            .unwrap();

        let completed = adapter.list_by_status(PlanStatus::Completed).await.unwrap();
        assert_eq!(completed.len(), 1);

        let pending = adapter.list_by_status(PlanStatus::Pending).await.unwrap();
        assert_eq!(pending.len(), 1);
    }

    #[tokio::test]
    async fn test_search() {
        let adapter = create_test_adapter();

        adapter
            .save_plan(&create_test_plan("search-1", "Deploy application"))
            .await
            .unwrap();
        adapter
            .save_plan(&create_test_plan("search-2", "Build project"))
            .await
            .unwrap();
        adapter
            .save_plan(&create_test_plan("search-3", "Deploy database"))
            .await
            .unwrap();

        let results = adapter.search("deploy").await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_stats() {
        let adapter = create_test_adapter();

        adapter
            .save_plan(&create_test_plan("stat-1", "Goal 1"))
            .await
            .unwrap();
        adapter
            .save_plan(&create_test_plan("stat-2", "Goal 2"))
            .await
            .unwrap();

        let result = create_test_result("stat-1");
        adapter.save_result(&result, "Goal 1").await.unwrap();

        let stats = adapter.stats().await.unwrap();
        assert_eq!(stats.total_plans, 2);
        assert_eq!(stats.completed_plans, 1);
        assert_eq!(stats.pending_plans, 1);
    }

    #[tokio::test]
    async fn test_with_prefix() {
        let storage = Arc::new(MemoryStorage::new());
        let adapter = TaskStorageAdapter::new(storage.clone()).with_prefix("custom");

        let plan = create_test_plan("prefix-test", "Prefix Goal");
        adapter.save_plan(&plan).await.unwrap();

        let exists = storage.exists("custom/plans/prefix-test").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_clear() {
        let adapter = create_test_adapter();

        adapter
            .save_plan(&create_test_plan("c1", "Clear 1"))
            .await
            .unwrap();
        adapter
            .save_plan(&create_test_plan("c2", "Clear 2"))
            .await
            .unwrap();

        let count = adapter.clear().await.unwrap();
        assert!(count >= 2);

        let plans = adapter.list_plans().await.unwrap();
        assert!(plans.is_empty());
    }

    #[tokio::test]
    async fn test_result_updates_plan_status() {
        let adapter = create_test_adapter();

        let plan = create_test_plan("auto-status", "Auto Status");
        adapter.save_plan(&plan).await.unwrap();

        // 成功结果
        let success_result = create_test_result("auto-status");
        adapter.save_result(&success_result, "Auto Status").await.unwrap();

        let loaded = adapter.load_plan("auto-status").await.unwrap();
        assert_eq!(loaded.status, PlanStatus::Completed);
    }

    #[tokio::test]
    async fn test_failed_result() {
        let adapter = create_test_adapter();

        let plan = create_test_plan("fail-test", "Fail Test");
        adapter.save_plan(&plan).await.unwrap();

        // 失败结果
        let failed_result = ExecutionResult {
            plan_id: "fail-test".to_string(),
            total_tasks: 2,
            completed_tasks: 1,
            failed_tasks: 1,
            skipped_tasks: 0,
            total_time: 5,
            task_results: vec![],
        };
        adapter.save_result(&failed_result, "Fail Test").await.unwrap();

        let loaded = adapter.load_plan("fail-test").await.unwrap();
        assert_eq!(loaded.status, PlanStatus::Failed);
    }
}
