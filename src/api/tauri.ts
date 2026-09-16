import { invoke, Channel } from "@tauri-apps/api/core";
import type { CommitInfo, CommitResult, FileDiff, ProjectInfo, Provider, RepoStatus } from "@/lib/types";
export async function addProject(path: string): Promise<ProjectInfo> {
  return invoke<ProjectInfo>("add_project", { path });
}

export async function removeProject(path: string): Promise<void> {
  await invoke("remove_project", { path });
}

export async function refreshProject(path: string): Promise<ProjectInfo> {
  return invoke<ProjectInfo>("refresh_project", { path });
}

export async function gitInit(path: string): Promise<string> {
  return invoke<string>("git_init", { path });
}

export async function listProjects(): Promise<ProjectInfo[]> {
  return invoke<ProjectInfo[]>("list_projects");
}

export async function getGitDiff(path: string, stagedOnly = false): Promise<string> {
  return invoke<string>("get_git_diff", { path, stagedOnly });
}

export async function gitStatus(path: string): Promise<RepoStatus> {
  return invoke<RepoStatus>("git_status", { path });
}

export async function gitLog(path: string, limit = 150): Promise<CommitInfo[]> {
  return invoke<CommitInfo[]>("git_log", { path, limit });
}

export async function gitShowCommit(path: string, hash: string): Promise<FileDiff[]> {
  return invoke<FileDiff[]>("git_show_commit", { path, hash });
}


export async function gitCommit(path: string, message: string): Promise<CommitResult> {
  return invoke<CommitResult>("git_commit", { path, message });
}

export async function gitPush(path: string): Promise<string> {
  return invoke<string>("git_push", { path });
}

export async function listProviders(): Promise<Provider[]> {
  return invoke<Provider[]>("list_providers");
}

export async function saveProvider(provider: Provider): Promise<void> {
  await invoke("save_provider", { provider });
}

export async function deleteProvider(id: string): Promise<void> {
  await invoke("delete_provider", { id });
}

export async function generateCommitMessage(path: string, providerId: string, note: string): Promise<string> {
  return invoke<string>("generate_commit_message", { path, providerId, note });
}

export type CommitStreamEvent =
  | { type: "chunk"; text: string }
  | { type: "done"; message: string }
  | { type: "error"; message: string };

export function streamCommitMessage(
  path: string,
  providerId: string,
  note: string,
  onEvent: (e: CommitStreamEvent) => void,
): Promise<void> {
  const channel = new Channel<CommitStreamEvent>();
  channel.onmessage = onEvent;
  return invoke("generate_commit_message_stream", { path, providerId, note, onEvent: channel });
}