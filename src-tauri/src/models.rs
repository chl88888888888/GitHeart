//! 分析结果的数据模型。

use serde::{Deserialize, Serialize};

/// 单个文件的统计信息。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileStats {
    /// 文件路径（相对于仓库根目录）。
    pub path: String,
    /// 总变更次数（影响该文件的提交数）。
    pub total_changes: u32,
    /// 添加的行数。
    pub lines_added: u32,
    /// 删除的行数。
    pub lines_deleted: u32,
    /// 流失指数 = (添加行数 + 删除行数) / 变更次数。
    /// 值越高表示单次改动越剧烈。
    pub churn_index: f64,
}

/// 仓库每月的变更量（流失）。
#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyChurn {
    /// 月份，格式为 YYYY-MM。
    pub month: String,
    /// 该月内的提交数量。
    pub changes: u32,
}

/// 完整的仓库分析结果。
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// 仓库路径（本地路径或 GitHub 全名）。
    pub repo_path: String,
    /// 分析的提交总数（上限为 500）。
    pub total_commits: usize,
    /// 每个文件的统计信息，按总变更次数降序排列。
    pub file_stats: Vec<FileStats>,
    /// 每月的提交数量，按时间顺序排列。
    pub monthly_churn: Vec<MonthlyChurn>,
    /// 基于历史数据的智能洞察列表。
    pub insights: Vec<String>,
}