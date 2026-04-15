import { invoke } from '@tauri-apps/api/core';
import { ref } from 'vue';

export interface FileStats {
  path: string;
  total_changes: number;
  lines_added: number;
  lines_deleted: number;
  churn_index: number;
}

export interface MonthlyChurn {
  month: string;
  changes: number;
}

export interface AnalysisResult {
  repo_path: string;
  total_commits: number;
  file_stats: FileStats[];
  monthly_churn: MonthlyChurn[];
  insights: string[];
}

export function useGitAnalyzer() {
  const loading = ref(false);
  const result = ref<AnalysisResult | null>(null);
  const error = ref<string | null>(null);

  const analyzeLocal = async (repoPath: string) => {
    loading.value = true;
    error.value = null;
    try {
      result.value = await invoke('analyze_local_repo', { repoPath });
    } catch (e: any) {
      error.value = e.toString();
    } finally {
      loading.value = false;
    }
  };

  const analyzeGitHub = async (url: string, token?: string) => {
    loading.value = true;
    error.value = null;
    try {
      result.value = await invoke('analyze_github_repo', { repoUrl: url, githubToken: token });
    } catch (e: any) {
      error.value = e.toString();
    } finally {
      loading.value = false;
    }
  };

  return { loading, result, error, analyzeLocal, analyzeGitHub };
}