export type TabId = "projects" | "history" | "providers";

export interface ChangedFile {
  index: string;
  worktree: string;
  path: string;
}

export interface RepoStatus {
  branch: string;
  ahead_behind: string;
  files: ChangedFile[];
  staged: number;
  unstaged: number;
  untracked: number;
  clean: boolean;
}

export interface ProjectInfo {
  path: string;
  name: string;
  branch: string;
  ahead_behind: string;
  changed: number;
  clean: boolean;
  error?: string | null;
}

export interface CommitInfo {
  hash: string;
  shortHash: string;
  subject: string;
  author: string;
  date: string;
  parents: string[];
}

export interface FileDiff {
  path: string;
  status: string;
  oldContent: string;
  newContent: string;
}

export interface CommitResult {
  hash: string;
  shortHash: string;
}

export interface Provider {
  id: string;
  name: string;
  base_url: string;
  api_key: string;
  model: string;
  system_prompt?: string | null;
  extra_headers: Record<string, string>;
}