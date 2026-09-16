import { reactive } from "vue";

export type Locale = "vi" | "en";

export type MessageKey =
  | "tabProjects"
  | "tabHistory"
  | "tabProviders"
  | "selectedProject"
  | "switchLocale"
  | "addInputPlaceholder"
  | "adding"
  | "addProject"
  | "gitInit"
  | "noProjectsYet"
  | "noProviderSelected"
  | "cleanRepo"
  | "statusLine"
  | "openHistory"
  | "loading"
  | "noChanges"
  | "aiProvider"
  | "noProviderOption"
  | "aiGenerating"
  | "generateWithAi"
  | "notePlaceholder"
  | "messagePlaceholder"
  | "aiDone"
  | "committed"
  | "footerHint"
  | "refresh"
  | "committing"
  | "commit"
  | "historyTitle"
  | "reload"
  | "noProjectsHint"
  | "loadingHistory"
  | "noCommits"
  | "commitsCount"
  | "loadingDiff"
  | "selectCommitPrompt"
  | "files"
  | "noFileChanges"
  | "fileAddedNote"
  | "fileDeletedNote"
  | "truncatedNote"
  | "noProviders"
  | "editProviderTitle"
  | "addProviderTitle"
  | "providerNameField"
  | "providerNamePlaceholder"
  | "modelField"
  | "modelPlaceholder"
  | "baseUrlField"
  | "baseUrlPlaceholder"
  | "apiKeyField"
  | "apiKeyPlaceholder"
  | "systemPromptField"
  | "systemPromptPlaceholder"
  | "providerValidationErr"
  | "providerSaved"
  | "saving"
  | "saveChanges"
  | "saveProvider"
  | "cancel"
  | "notAGitRepo"
  | "notAGitRepoDesc"
  | "initGitButton"
  | "initializing"
  | "removeProject"
  | "initSuccess"
  | "noGitBadge"
  | "copyCommitId"
  | "copyCommitMsg"
  | "copiedCommitId"
  | "copiedCommitMsg";

const vi: Record<MessageKey, string> = {
  tabProjects: "Dự án",
  tabHistory: "Lịch sử",
  tabProviders: "AI Providers",
  selectedProject: "Đang chọn",
  switchLocale: "EN",
  addInputPlaceholder: "Nhập đường dẫn thư mục git...",
  adding: "Đang thêm...",
  addProject: "Thêm dự án",
  gitInit: "Khởi tạo Git",
  noProjectsYet: "Chưa có dự án nào. Nhập đường dẫn ở trên để thêm.",
  noProviderSelected: "Chưa chọn provider",
  cleanRepo: "Sạch",
  statusLine: "{staged} staged / {unstaged} sửa / {untracked} mới",
  openHistory: "Lịch sử",
  loading: "Đang tải...",
  noChanges: "Không có thay đổi nào trong working directory.",
  aiProvider: "AI Provider",
  noProviderOption: "Chưa có provider — qua tab AI Providers",
  aiGenerating: "AI đang tạo...",
  generateWithAi: "Tạo commit với AI",
  notePlaceholder: "Ghi chú cho AI (tuỳ chọn)...",
  messagePlaceholder: "Commit message sẽ hiện ở đây. Bạn có thể sửa trước khi commit...",
  aiDone: "Message đã được AI tạo. Duyệt lại rồi bấm Commit.",
  committed: "Đã commit {hash}",
  footerHint: "AI tạo message dạng conventional commit, bạn duyệt rồi bấm Commit.",
  refresh: "Làm mới",
  committing: "Đang commit...",
  commit: "Commit",
  historyTitle: "Lịch sử commit",
  reload: "Tải lại",
  noProjectsHint: "Chưa có dự án nào — thêm dự án ở tab Dự án để xem lịch sử commit.",
  loadingHistory: "Đang tải lịch sử commit...",
  noCommits: "Repository chưa có commit nào.",
  commitsCount: "Commits ({count})",
  loadingDiff: "Đang tải diff...",
  selectCommitPrompt: "Chọn một commit ở bên trái.",
  files: "Files",
  noFileChanges: "Commit này không thay đổi file nào.",
  fileAddedNote: "File mới — không có nội dung cũ.",
  fileDeletedNote: "File đã bị xóa — không có nội dung mới.",
  truncatedNote: "... cắt ở {lines} dòng (tổng {total})",
  noProviders: "Chưa có AI provider nào.",
  editProviderTitle: "Sửa provider {name}",
  addProviderTitle: "Thêm AI provider (OpenAI-compatible /chat/completions)",
  providerNameField: "Tên provider",
  providerNamePlaceholder: "VD: DeepSeek của tôi, OpenAI...",
  modelField: "Model",
  modelPlaceholder: "VD: deepseek-chat",
  baseUrlField: "Base URL (chat completions base)",
  baseUrlPlaceholder: "VD: https://api.deepseek.com/v1",
  apiKeyField: "API Key",
  apiKeyPlaceholder: "sk-...",
  systemPromptField: "System prompt (tuỳ chọn)",
  systemPromptPlaceholder:
    "Mặc định sẽ dùng prompt giống cline (generate informative git commit messages based on git diffs)...",
  providerValidationErr: "Tên, Base URL và Model không được để trống",
  providerSaved: "Đã lưu provider",
  saving: "Đang lưu...",
  saveChanges: "Lưu thay đổi",
  saveProvider: "Lưu provider",
  cancel: "Huỷ",
  notAGitRepo: "Chưa khởi tạo Git",
  notAGitRepoDesc: "Thư mục này chưa phải là Git repository. Khởi tạo Git để bắt đầu theo dõi thay đổi và tạo commit với AI.",
  initGitButton: "Khởi tạo Git repository (git init)",
  initializing: "Đang khởi tạo Git...",
  removeProject: "Xóa khỏi danh sách",
  initSuccess: "Đã khởi tạo Git repository thành công!",
  noGitBadge: "Chưa có Git",
  copyCommitId: "Sao chép commit ID (hash)",
  copyCommitMsg: "Sao chép commit message",
  copiedCommitId: "Đã chép commit ID",
  copiedCommitMsg: "Đã chép message",
};

const en: Record<MessageKey, string> = {
  tabProjects: "Projects",
  tabHistory: "History",
  tabProviders: "AI Providers",
  selectedProject: "Selected",
  switchLocale: "VI",
  addInputPlaceholder: "Enter git folder path...",
  adding: "Adding...",
  addProject: "Add project",
  gitInit: "Git init",
  noProjectsYet: "No projects yet. Enter a path above to add one.",
  noProviderSelected: "No provider selected",
  cleanRepo: "Clean",
  statusLine: "{staged} staged / {unstaged} modified / {untracked} untracked",
  openHistory: "History",
  loading: "Loading...",
  noChanges: "No changes in the working directory.",
  aiProvider: "AI Provider",
  noProviderOption: "No provider — go to the AI Providers tab",
  aiGenerating: "AI is generating...",
  generateWithAi: "Generate AI commit",
  notePlaceholder: "Note for the AI (optional)...",
  messagePlaceholder: "Commit message will appear here. Edit before committing...",
  aiDone: "Message generated by AI. Review it, then Commit.",
  committed: "Committed {hash}",
  footerHint: "AI writes conventional-commit style messages; review, then Commit.",
  refresh: "Refresh",
  committing: "Committing...",
  commit: "Commit",
  historyTitle: "Commit history",
  reload: "Reload",
  noProjectsHint: "No projects yet — add one in the Projects tab to view commit history.",
  loadingHistory: "Loading commit history...",
  noCommits: "Repository has no commits yet.",
  commitsCount: "Commits ({count})",
  loadingDiff: "Loading diff...",
  selectCommitPrompt: "Select a commit on the left.",
  files: "Files",
  noFileChanges: "This commit changed no files.",
  fileAddedNote: "New file — no old content.",
  fileDeletedNote: "File deleted — no new content.",
  truncatedNote: "... truncated at {lines} lines ({total} total)",
  noProviders: "No AI provider yet.",
  editProviderTitle: "Edit provider {name}",
  addProviderTitle: "Add AI provider (OpenAI-compatible /chat/completions)",
  providerNameField: "Provider name",
  providerNamePlaceholder: "e.g. My DeepSeek, OpenAI...",
  modelField: "Model",
  modelPlaceholder: "e.g. deepseek-chat",
  baseUrlField: "Base URL (chat completions base)",
  baseUrlPlaceholder: "e.g. https://api.deepseek.com/v1",
  apiKeyField: "API Key",
  apiKeyPlaceholder: "sk-...",
  systemPromptField: "System prompt (optional)",
  systemPromptPlaceholder:
    "Defaults to the cline-style prompt (generate informative git commit messages based on git diffs)...",
  providerValidationErr: "Name, Base URL and Model are required",
  providerSaved: "Provider saved",
  saving: "Saving...",
  saveChanges: "Save changes",
  saveProvider: "Save provider",
  cancel: "Cancel",
  notAGitRepo: "Not a Git Repository",
  notAGitRepoDesc: "This directory is not a Git repository yet. Initialize Git to start tracking changes and generating commit messages with AI.",
  initGitButton: "Initialize Git repository (git init)",
  initializing: "Initializing Git...",
  removeProject: "Remove from list",
  initSuccess: "Git repository initialized successfully!",
  noGitBadge: "No Git",
  copyCommitId: "Copy commit ID (hash)",
  copyCommitMsg: "Copy commit message",
  copiedCommitId: "Copied commit ID",
  copiedCommitMsg: "Copied message",
};

const messages: Record<Locale, Record<MessageKey, string>> = { vi, en };

function detectLocale(): Locale {
  const stored = localStorage.getItem("commitgen.locale");
  if (stored === "vi" || stored === "en") return stored;
  return (navigator.language || "").toLowerCase().startsWith("vi") ? "vi" : "en";
}

export const i18n = reactive({ locale: detectLocale() as Locale });

export function t(key: MessageKey, params?: Record<string, string | number>): string {
  let out = messages[i18n.locale][key] ?? messages.en[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      out = out.replaceAll(`{${k}}`, String(v));
    }
  }
  return out;
}

export function setLocale(locale: Locale) {
  i18n.locale = locale;
  localStorage.setItem("commitgen.locale", locale);
}