use crate::models::{AnalysisResult, FileStats, MonthlyChurn, CommitSample};
use chrono::{TimeZone, Utc};
use git2::{Repository, Sort};
use std::cell::RefCell;
use std::collections::HashMap;

/// 分析本地 Git 仓库
///
/// 打开 `repo_path` 处的仓库，遍历提交历史（最多 500 个提交），
/// 统计每个源文件的变更次数、增删行数，并聚合每月的提交数量
///
/// # 参数
///
/// * `repo_path` - 本地 Git 仓库的路径
///
/// # 返回值
///
/// 包含总提交数、文件统计信息、每月流失数据和智能洞察的 `AnalysisResult`
///
/// # 错误
///
/// 若路径不是有效的 Git 仓库，或遍历提交时发生错误，则返回包含错误信息的 `String`
#[tauri::command]
pub fn analyze_local_repo(repo_path: String) -> Result<AnalysisResult, String> {
    let repo = Repository::open(&repo_path).map_err(|e| format!("不是有效的 Git 仓库: {}", e))?;

    let mut revwalk = repo
        .revwalk()
        .map_err(|e| format!("无法创建提交遍历器: {}", e))?;
    revwalk
        .push_head()
        .map_err(|e| format!("无法获取 HEAD: {}", e))?;
    revwalk
        .set_sorting(Sort::TIME)
        .map_err(|e| format!("设置排序失败: {}", e))?;

    // 文件路径 -> (变更次数, 添加行数, 删除行数)
    let file_change_count = RefCell::new(HashMap::new());
    let current_file = RefCell::new(None);
    let mut monthly_changes: HashMap<String, u32> = HashMap::new();
    let mut total_commits = 0;
    let mut commit_samples = Vec::new();
    const MAX_COMMITS: usize = 500;

    for oid in revwalk {
        if total_commits >= MAX_COMMITS {
            break;
        }
        let oid = oid.map_err(|e| format!("无效的 commit ID: {}", e))?;
        let commit = repo
            .find_commit(oid)
            .map_err(|e| format!("找不到 commit: {}", e))?;

        total_commits += 1;

        // 月度统计
        let timestamp = commit.time().seconds();
        let datetime = Utc
            .timestamp_opt(timestamp, 0)
            .single()
            .ok_or("无效的时间戳")?;
        let month_key = datetime.format("%Y-%m").to_string();
        *monthly_changes.entry(month_key).or_insert(0) += 1;

        // 用于统计本次提交的数据
        let local_data = RefCell::new((0u32, 0u32, 0u32)); // (added, deleted, file_count)

        // 获取与父提交的 diff 并统计文件变更
        if let Ok(parent) = commit.parent(0) {
            let parent_tree = parent
                .tree()
                .map_err(|e| format!("获取父提交树失败: {}", e))?;
            let commit_tree = commit
                .tree()
                .map_err(|e| format!("获取提交树失败: {}", e))?;

            let mut diff = repo
                .diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)
                .map_err(|e| format!("生成 diff 失败: {}", e))?;

            // 需要检测重命名等相似性，以获得更准确的文件路径
            diff.find_similar(None)
                .map_err(|e| format!("查找相似文件失败: {}", e))?;

            diff.foreach(
                &mut |delta, _| {
                    // 文件回调：判断是否为源码文件，并记录路径
                    if let Some(file) = delta.new_file().path() {
                        let path = file.to_string_lossy().to_string();
                        if is_source_file(&path) {
                            // 该文件在此次提交中被修改，变更次数 +1
                            let mut map = file_change_count.borrow_mut();
                            let entry = map.entry(path.clone()).or_insert((0, 0, 0));
                            entry.0 += 1;
                            *current_file.borrow_mut() = Some(path);
                            // 文件计数 +1
                            local_data.borrow_mut().2 += 1;
                        } else {
                            *current_file.borrow_mut() = None;
                        }
                    } else {
                        *current_file.borrow_mut() = None;
                    }
                    true
                },
                None, // binary 回调（忽略）
                Some(&mut |_hunk, _| true), // hunk 回调（不需处理，返回 true 继续）
                Some(&mut |_, _, line| {
                    // 行回调：统计增删行数
                    if let Some(path) = current_file.borrow().as_ref() {
                        let mut map = file_change_count.borrow_mut();
                        // 理论上文件回调中已创建条目，这里直接获取
                        let entry = map
                            .get_mut(path)
                            .expect("文件条目应在文件回调中已创建");
                        match line.origin() {
                            '+' => {
                                entry.1 += 1;
                                local_data.borrow_mut().0 += 1;
                            }
                            '-' => {
                                entry.2 += 1;
                                local_data.borrow_mut().1 += 1;
                            }
                            _ => {}
                        }
                    }
                    true
                }),
            )
            .map_err(|e| format!("遍历 diff 失败: {}", e))?;
        }

        let (added, deleted, files) = local_data.into_inner();
        commit_samples.push(CommitSample {
            timestamp: datetime,
            added_lines: added,
            deleted_lines: deleted,
            file_count: files,
            total_churn: added + deleted,
        });
    }

    let file_change_count = file_change_count.into_inner();

    // 构建文件统计结果
    let mut file_stats: Vec<FileStats> = file_change_count
        .into_iter()
        .map(|(path, (changes, added, deleted))| {
            let churn_index = if changes > 0 {
                (added + deleted) as f64 / changes as f64
            } else {
                0.0
            };
            FileStats {
                path,
                total_changes: changes,
                lines_added: added,
                lines_deleted: deleted,
                churn_index,
            }
        })
        .collect();
    file_stats.sort_by(|a, b| b.total_changes.cmp(&a.total_changes));

    // 构建月度流失数据
    let mut monthly_churn: Vec<MonthlyChurn> = monthly_changes
        .into_iter()
        .map(|(month, changes)| MonthlyChurn { month, changes })
        .collect();
    monthly_churn.sort_by(|a, b| a.month.cmp(&b.month));

    // 生成智能洞察
    let insights = generate_insights(&file_stats, total_commits, &monthly_churn);

    Ok(AnalysisResult {
        repo_path,
        total_commits,
        file_stats,
        monthly_churn,
        insights,
        commit_samples,
    })
}

/// 判断一个文件路径是否对应源代码或配置文件
///
/// 检查文件扩展名是否在已知的源码/配置扩展名列表中，
/// 同时匹配特殊文件名（如 `Dockerfile` 或 `Makefile`）
fn is_source_file(path: &str) -> bool {
    let special_files = [
        "dockerfile",
        "makefile",
        "jenkinsfile",
        "vagrantfile",
        "license",
        "readme",
        "changelog",
        "cargo.toml",
        "package.json",
        "go.mod",
        "requirements.txt",
        "pipfile",
        "gemfile",
        "cmakelists.txt",
        ".gitignore",
        ".dockerignore",
        ".env",
        ".editorconfig",
    ];
    let file_name = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let file_name_lower = file_name.to_lowercase();
    if special_files.contains(&file_name_lower.as_str()) {
        return true;
    }

    let extensions = [
        "rs", "py", "js", "ts", "jsx", "tsx", "vue", "java", "kt", "go", "c", "cpp", "h", "hpp",
        "cs", "rb", "php", "swift", "m", "mm", "scala", "sh", "yaml", "yml", "toml", "json",
        "html", "css", "scss", "sass", "less", "sql", "md", "txt", "xml", "gradle",
    ];
    if let Some(ext) = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
    {
        extensions.contains(&ext.as_str())
    } else {
        false
    }
}

/// 基于分析数据生成智能洞察文本
///
/// 包括：
/// - 上帝文件（修改频率过高）
/// - 高流失文件（单次改动剧烈）
/// - 配置文件频繁变动
/// - 提交集中度异常
fn generate_insights(
    file_stats: &[FileStats],
    total_commits: usize,
    monthly_churn: &[MonthlyChurn],
) -> Vec<String> {
    let mut insights = Vec::new();

    if total_commits == 0 {
        return insights;
    }

    // 1. 上帝文件检测
    if let Some(god_file) = file_stats.first() {
        let god_ratio = god_file.total_changes as f64 / total_commits as f64;
        if god_ratio > 0.2 {
            insights.push(format!(
                "上帝文件：'{}' 占全部提交的 {:.1}%，修改极其频繁，耦合度高",
                god_file.path,
                god_ratio * 100.0
            ));
        }
    }

    // 2. 高流失文件检测 (改动剧烈)
    let high_churn_files: Vec<_> = file_stats
        .iter()
        .filter(|f| f.churn_index > 100.0 && f.total_changes > 5)
        .take(3)
        .collect();
    if !high_churn_files.is_empty() {
        let names: Vec<_> = high_churn_files.iter().map(|f| f.path.as_str()).collect();
        insights.push(format!(
            "代码变更剧烈：{} 等文件单次改动涉及大量增删，架构可能不稳定",
            names.join("、")
        ));
    }

    // 3. 配置文件频繁变动
    let config_files: Vec<_> = file_stats
        .iter()
        .filter(|f| {
            f.path.contains("Cargo.toml")
                || f.path.contains("package.json")
                || f.path.contains("go.mod")
                || f.path.contains("requirements.txt")
        })
        .filter(|f| f.total_changes > total_commits as u32 / 10)
        .collect();
    if !config_files.is_empty() {
        insights.push("依赖配置文件变更频繁，请注意版本兼容性风险".to_string());
    }

    // 4. 提交集中度警告
    if monthly_churn.len() >= 3 {
        if let Some(max_month) = monthly_churn.iter().max_by_key(|m| m.changes) {
            let avg_other: u32 = monthly_churn
                .iter()
                .filter(|m| m.month != max_month.month)
                .map(|m| m.changes)
                .sum::<u32>()
                / (monthly_churn.len() - 1) as u32;
            if max_month.changes > avg_other * 3 {
                insights.push(format!(
                    "开发节奏异常：{} 月提交量 ({}) 是其他月份的 {} 倍，可能存在突击开发，注意质量",
                    max_month.month,
                    max_month.changes,
                    max_month.changes / avg_other
                ));
            }
        }
    }

    insights
}