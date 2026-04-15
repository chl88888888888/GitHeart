use crate::models::{AnalysisResult, FileStats, MonthlyChurn, CommitSample};
use octocrab::Octocrab;
use std::collections::HashMap;
use thiserror::Error;

/// GitHub 分析过程中可能出现的错误
#[derive(Error, Debug)]
pub enum AppError {
    /// 提供的 URL 不是有效的 GitHub 仓库地址
    #[error("无效的 GitHub 仓库地址: {0}")]
    InvalidUrl(String),

    /// GitHub API 返回的错误
    #[error("GitHub API 请求失败: {0}")]
    GithubApi(#[from] octocrab::Error),

    /// 读取环境变量（例如 `GITHUB_TOKEN`）失败
    #[error("环境变量读取失败: {0}")]
    EnvVar(#[from] std::env::VarError),

    /// API 调用在重试后仍然失败，可能是由于速率限制或网络问题
    #[error("网络请求超时或重试次数耗尽: {0}")]
    RetryFailed(String),

    /// 解析 API 响应失败
    #[error("解析响应数据失败: {0}")]
    ParseError(String),
}

impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

/// 将 GitHub URL 解析为所有者（owner）和仓库名（repo）
///
/// 支持的格式：
/// - `https://github.com/owner/repo`
/// - `http://github.com/owner/repo`
/// - `github.com/owner/repo`
/// - `owner/repo`（可选的 `.git` 后缀）
///
/// # 参数
///
/// * `url` - GitHub URL 或 `owner/repo` 字符串
///
/// # 返回值
///
/// 成功时返回 `(owner, repo)` 元组
fn parse_github_url(url: &str) -> Result<(String, String), AppError> {
    let trimmed = url
        .trim()
        .strip_prefix("https://github.com/")
        .or_else(|| url.trim().strip_prefix("http://github.com/"))
        .or_else(|| url.trim().strip_prefix("github.com/"))
        .unwrap_or(url.trim());

    let trimmed = trimmed.strip_suffix(".git").unwrap_or(trimmed);

    let parts: Vec<&str> = trimmed.split('/').collect();
    if parts.len() >= 2 {
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else {
        Err(AppError::InvalidUrl(url.to_string()))
    }
}

/// 判断文件路径是否应视为源代码或配置文件
///
/// 检查扩展名以及特殊文件名（不区分大小写）
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
        "html", "css", "scss", "sass", "less", "sql", "md", "txt", "xml", "gradle", "scala",
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

/// 为 API 调用包装自动重试机制
///
/// 最多重试 3 次如果检测到速率限制错误，则等待 60 秒；
/// 否则每次尝试间隔 2 秒
///
/// # 类型参数
///
/// * `F` - 返回执行 API 调用的 future 的闭包
/// * `Fut` - 闭包返回的 future 类型
/// * `T` - 成功时的结果类型
async fn retry_api_call<F, Fut, T>(mut f: F) -> Result<T, AppError>
where
    F: FnMut() -> Fut + Send,
    Fut: std::future::Future<Output = Result<T, octocrab::Error>> + Send,
    T: Send,
{
    let max_retries = 3;
    let mut attempt = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                eprintln!("请求失败 (尝试 {}/{}): {:?}", attempt + 1, max_retries, e);
                attempt += 1;
                if attempt >= max_retries {
                    return Err(AppError::RetryFailed(format!(
                        "API 调用失败（已重试 {} 次）: {}",
                        max_retries, e
                    )));
                }
                if e.to_string().contains("API rate limit exceeded") {
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        }
    }
}

/// 分页获取仓库的所有提交（最多 500 条）
async fn fetch_all_commits(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
) -> Result<Vec<octocrab::models::repos::RepoCommit>, AppError> {
    let mut all_commits = Vec::new();
    let mut page_num = 1u32;
    const PER_PAGE: u8 = 100;
    const MAX_COMMITS: usize = 500;
    const MAX_PAGES: u32 = 10;

    while all_commits.len() < MAX_COMMITS && page_num <= MAX_PAGES {
        let page = retry_api_call(|| async {
            octocrab
                .repos(owner, repo)
                .list_commits()
                .per_page(PER_PAGE)
                .page(page_num)
                .send()
                .await
        })
        .await?;

        let items = page.items;
        if items.is_empty() {
            break;
        }

        let remaining = MAX_COMMITS - all_commits.len();
        let take = items.len().min(remaining);
        all_commits.extend(items.into_iter().take(take));

        if page.next.is_none() || all_commits.len() >= MAX_COMMITS {
            break;
        }
        page_num += 1;
    }

    Ok(all_commits)
}

/// 使用 GitHub API 分析一个 GitHub 仓库
///
/// 解析仓库 URL，获取提交数据（最多 500 个提交），
/// 统计每个源文件的变更次数，并聚合每月的提交数量
///
/// # 参数
///
/// * `repo_url` - 完整的 GitHub URL（例如 `https://github.com/owner/repo`）或 `owner/repo` 格式
/// * `github_token` - 可选的 GitHub Personal Access Token（提升速率限制至 5000 次/小时）
///
/// # 返回值
///
/// 包含总提交数、文件统计信息和每月流失数据的 `AnalysisResult`
///
/// # 错误
///
/// 若 URL 解析失败、API 请求出错或重试耗尽，则返回错误信息
#[tauri::command]
pub async fn analyze_github_repo(
    repo_url: String,
    github_token: Option<String>,
) -> Result<AnalysisResult, String> {
    let (owner, repo) = parse_github_url(&repo_url)?;

    // 构建 Octocrab 客户端，优先使用前端传入的 token
    let octocrab = {
        let mut builder = Octocrab::builder();
        if let Some(token) = github_token {
            builder = builder.personal_token(token);
        }
        builder
            .build()
            .map_err(|e| AppError::GithubApi(e).to_string())?
    };

    // 获取仓库基本信息
    let repo_info = retry_api_call(|| async { octocrab.repos(&owner, &repo).get().await }).await?;

    // 获取提交历史
    let commits = fetch_all_commits(&octocrab, &owner, &repo).await?;
    let total_commits = commits.len();

    let mut monthly_changes: HashMap<String, u32> = HashMap::new();
    // 存储：文件路径 -> (变更次数, 添加行数, 删除行数)
    let mut file_change_count: HashMap<String, (u32, u32, u32)> = HashMap::new();
    let mut commit_samples = Vec::new();

    for commit in commits.iter() {
        // 统计月度提交数
        if let Some(Some(date)) = commit.commit.committer.clone().map(|c| c.date) {
            let month_key = date.format("%Y-%m").to_string();
            *monthly_changes.entry(month_key).or_insert(0) += 1;
        }

        // 获取每个提交的详细文件变更
        let sha = commit.sha.clone();
        let detailed_commit = retry_api_call(|| async {
            octocrab.commits(&owner, &repo).get(&sha).await
        })
        .await?;

        let mut added = 0;
        let mut deleted = 0;
        let mut file_count = 0;

        if let Some(files) = detailed_commit.files {
            for file in files {
                if is_source_file(&file.filename) {
                    file_count += 1;
                    added += file.additions as u32;
                    deleted += file.deletions as u32;

                    let entry = file_change_count
                        .entry(file.filename)
                        .or_insert((0, 0, 0));
                    entry.0 += 1;
                    entry.1 += file.additions as u32;
                    entry.2 += file.deletions as u32;
                }
            }
        }

        // 提取提交时间
        let timestamp = commit
            .commit
            .committer
            .as_ref()
            .and_then(|c| c.date)
            .unwrap_or_else(chrono::Utc::now);

        commit_samples.push(CommitSample {
            timestamp,
            added_lines: added,
            deleted_lines: deleted,
            file_count,
            total_churn: added + deleted,
        });
    }

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
        repo_path: repo_info
            .full_name
            .unwrap_or_else(|| format!("{}/{}", owner, repo)),
        total_commits,
        file_stats,
        monthly_churn,
        insights,
        commit_samples,
    })
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