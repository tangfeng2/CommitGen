mod config;
mod git;
mod providers;

use std::sync::Mutex;
use tauri::{Manager, State};

pub struct AppState {
    pub config: Mutex<config::AppConfig>,
    pub config_path: std::path::PathBuf,
}

impl AppState {
    fn save_config(&self) -> Result<(), String> {
        let cfg = self.config.lock().unwrap();
        config::save(&cfg, &self.config_path)
    }
}

fn project_info(path: &str) -> Result<config::ProjectInfo, String> {
    let root = git::repo_root(path)?;
    let status = git::git_status(&root)?;
    Ok(config::ProjectInfo {
        path: root.clone(),
        name: git::repo_name(&root),
        branch: status.branch.clone(),
        ahead_behind: status.ahead_behind.clone(),
        changed: status.staged + status.unstaged + status.untracked,
        clean: status.clean,
        error: None,
    })
}

#[tauri::command]
fn add_project(state: State<'_, AppState>, path: String) -> Result<config::ProjectInfo, String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(format!("Path does not exist: {path}"));
    }
    let norm = p.canonicalize().map(|cp| {
        let s = cp.to_string_lossy().to_string();
        s.strip_prefix(r"\\?\").unwrap_or(&s).replace('\\', "/")
    }).unwrap_or_else(|_| path.replace('\\', "/"));

    let root = match git::repo_root(&norm) {
        Ok(r) => r,
        Err(_) => norm.clone(),
    };
    {
        let mut cfg = state.config.lock().unwrap();
        if !cfg.projects.contains(&root) {
            cfg.projects.push(root.clone());
        }
    }
    state.save_config()?;
    match project_info(&root) {
        Ok(info) => Ok(info),
        Err(err) => Ok(config::ProjectInfo {
            path: root.clone(),
            name: config::name_of(&root),
            branch: String::new(),
            ahead_behind: String::new(),
            changed: 0,
            clean: false,
            error: Some(err),
        }),
    }
}

#[tauri::command]
fn remove_project(state: State<'_, AppState>, path: String) -> Result<(), String> {
    {
        let mut cfg = state.config.lock().unwrap();
        cfg.projects.retain(|p| p != &path);
    }
    state.save_config()
}

#[tauri::command]
fn git_init(path: String) -> Result<String, String> {
    git::git_init(&path)
}

#[tauri::command]
fn refresh_project(path: String) -> Result<config::ProjectInfo, String> {
    project_info(&path)
}

#[tauri::command]
fn list_projects(state: State<'_, AppState>) -> Result<Vec<config::ProjectInfo>, String> {
    let cfg = state.config.lock().unwrap();
    let mut infos = Vec::new();
    for p in &cfg.projects {
        match project_info(p) {
            Ok(info) => infos.push(info),
            Err(err) => infos.push(config::ProjectInfo {
                path: p.clone(),
                name: config::name_of(p),
                branch: String::new(),
                ahead_behind: String::new(),
                changed: 0,
                clean: false,
                error: Some(err),
            }),
        }
    }
    Ok(infos)
}

#[tauri::command]
async fn get_git_diff(path: String, staged_only: bool) -> Result<String, String> {
    // git may block; run off the async runtime
    tauri::async_runtime::spawn_blocking(move || git::git_diff(&path, staged_only))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn git_status(path: String) -> Result<git::RepoStatus, String> {
    tauri::async_runtime::spawn_blocking(move || git::git_status(&path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn git_log(path: String, limit: usize) -> Result<Vec<git::Commit>, String> {
    tauri::async_runtime::spawn_blocking(move || git::git_log(&path, limit))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn git_show_commit(path: String, hash: String) -> Result<Vec<git::FileDiff>, String> {
    tauri::async_runtime::spawn_blocking(move || git::git_show_commit(&path, &hash))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn git_commit(path: String, message: String) -> Result<git::CommitResult, String> {
    tauri::async_runtime::spawn_blocking(move || git::git_commit(&path, &message))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn git_push(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git::git_push(&path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn list_providers(state: State<'_, AppState>) -> Result<Vec<providers::Provider>, String> {
    let cfg = state.config.lock().unwrap();
    Ok(cfg.providers.clone())
}

#[tauri::command]
fn save_provider(state: State<'_, AppState>, provider: providers::Provider) -> Result<(), String> {
    if provider.name.trim().is_empty() || provider.base_url.trim().is_empty() || provider.model.trim().is_empty() {
        return Err("Name, Base URL and Model cannot be empty".into());
    }
    {
        let mut cfg = state.config.lock().unwrap();
        let exists = cfg.providers.iter().any(|p| p.id == provider.id);
        if exists {
            for p in cfg.providers.iter_mut() {
                if p.id == provider.id {
                    *p = provider.clone();
                }
            }
        } else {
            cfg.providers.push(provider);
        }
    }
    state.save_config()
}

#[tauri::command]
fn delete_provider(state: State<'_, AppState>, id: String) -> Result<(), String> {
    {
        let mut cfg = state.config.lock().unwrap();
        cfg.providers.retain(|p| p.id != id);
    }
    state.save_config()
}

#[tauri::command]
async fn generate_commit_message(
    state: State<'_, AppState>,
    path: String,
    provider_id: String,
    note: String,
) -> Result<String, String> {
    let provider = {
        let cfg = state.config.lock().unwrap();
        cfg.providers.iter().find(|p| p.id == provider_id).cloned()
    };
    let provider = provider.ok_or_else(|| "AI provider not found, please check the Providers tab".to_string())?;

    let diff = { git::git_diff(&path, false) }.map_err(|e| {
        if e.contains("No changes") {
            "No changes in repository to generate commit message".to_string()
        } else {
            e
        }
    })?;

    providers::generate_commit_message(&provider, &diff, &note).await
}

#[tauri::command]
async fn generate_commit_message_stream(
    state: State<'_, AppState>,
    path: String,
    provider_id: String,
    note: String,
    on_event: tauri::ipc::Channel<providers::CommitStreamEvent>,
) -> Result<(), String> {
    let provider = {
        let cfg = state.config.lock().unwrap();
        cfg.providers.iter().find(|p| p.id == provider_id).cloned()
    };
    let provider = provider.ok_or_else(|| "AI provider not found, please check the Providers tab".to_string())?;

    let diff = { git::git_diff(&path, false) }.map_err(|e| {
        if e.contains("No changes") {
            "No changes in repository to generate commit message".to_string()
        } else {
            e
        }
    })?;

    providers::stream_commit_message(&provider, &diff, &note, on_event).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let config_path = config::config_path(app.handle())?;
            let config = config::load(app.handle());
            app.manage(AppState { config: Mutex::new(config), config_path });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_project,
            remove_project,
            refresh_project,
            git_init,
            list_projects,
            get_git_diff,
            git_status,
            git_log,
            git_show_commit,
            git_commit,
            git_push,
            list_providers,
            save_provider,
            delete_provider,
            generate_commit_message,
            generate_commit_message_stream,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}