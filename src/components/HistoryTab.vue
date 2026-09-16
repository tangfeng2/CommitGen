<script setup lang="ts">
import { computed, onMounted, ref, shallowRef, watch } from "vue";
import { DiffView, DiffModeEnum, getLang } from "@git-diff-view/vue";
import "@git-diff-view/vue/styles/diff-view.css";
import { generateDiffFile, type DiffFile } from "@git-diff-view/file";
import { getDiffViewHighlighter } from "@git-diff-view/shiki";
import Button from "@/components/ui/Button.vue";
import * as api from "@/api/tauri";
import type { CommitInfo, FileDiff } from "@/lib/types";
import { t } from "@/lib/i18n";

const props = defineProps<{
  projectPath: string | null;
  projects: { path: string; name: string }[];
}>();

const emit = defineEmits<{
  select: [path: string];
}>();

const commits = ref<CommitInfo[]>([]);
const selectedHash = ref<string | null>(null);
const diffs = ref<FileDiff[]>([]);
const activePath = ref<string | null>(null);
const loading = ref(false);
const loadingDiff = ref(false);
const errorMsg = ref("");

// Diff viewer state
const currentDiffFile = shallowRef<DiffFile | null>(null);
const highlighterInstance = shallowRef<any>(null);
const isBuildingDiff = ref(false);
const diffMode = ref<DiffModeEnum>(DiffModeEnum.SplitGitHub);
const wrapLines = ref(false);
const isCollapsed = ref(false);
const isAllExpanded = ref(false);
const copied = ref(false);
let copyTimer: any = null;

let highlighterPromise: Promise<any> | null = null;
async function getHighlighter() {
  if (!highlighterPromise) {
    highlighterPromise = getDiffViewHighlighter();
  }
  return highlighterPromise;
}

const effectivePath = computed(() => props.projectPath ?? props.projects[0]?.path ?? null);
const effectiveName = computed(() => props.projects.find((p) => p.path === effectivePath.value)?.name ?? null);

const selectedCommit = computed(() => commits.value.find((c) => c.hash === selectedHash.value) ?? null);
const activeDiff = computed(() => diffs.value.find((d) => d.path === activePath.value) ?? null);

const isBinary = computed(() => {
  const d = activeDiff.value;
  if (!d) return false;
  return d.oldContent === "[binary file]" || d.newContent === "[binary file]";
});

const statusLabel: Record<string, string> = { A: "Added", M: "Modified", D: "Deleted" };

function statusTint(status: string) {
  return status === "A" ? "text-emerald-400" : status === "D" ? "text-rose-400" : "text-amber-400";
}

function fmtDate(iso: string) {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? iso : d.toLocaleString();
}

function copyPath(p: string) {
  navigator.clipboard.writeText(p);
  copied.value = true;
  clearTimeout(copyTimer);
  copyTimer = setTimeout(() => {
    copied.value = false;
  }, 1500);
}

const copiedType = ref<string | null>(null);
let commitCopyTimer: any;

function copyText(text: string, type: string) {
  navigator.clipboard.writeText(text);
  copiedType.value = type;
  clearTimeout(commitCopyTimer);
  commitCopyTimer = setTimeout(() => {
    if (copiedType.value === type) {
      copiedType.value = null;
    }
  }, 1800);
}

function toggleExpandAll() {
  if (!currentDiffFile.value) return;
  const mode = diffMode.value === DiffModeEnum.Unified ? "unified" : "split";
  if (isAllExpanded.value) {
    currentDiffFile.value.onAllCollapse(mode);
    isAllExpanded.value = false;
  } else {
    currentDiffFile.value.onAllExpand(mode);
    isAllExpanded.value = true;
  }
}

function resolveLang(filePath: string): string {
  const ext = (filePath.split(".").pop() || "").toLowerCase();
  const map: Record<string, string> = {
    rs: "rust",
    ts: "typescript",
    tsx: "tsx",
    js: "javascript",
    jsx: "jsx",
    vue: "vue",
    py: "python",
    go: "go",
    java: "java",
    c: "c",
    cpp: "cpp",
    cc: "cpp",
    cxx: "cpp",
    h: "c",
    hpp: "cpp",
    cs: "csharp",
    html: "html",
    css: "css",
    scss: "scss",
    less: "less",
    json: "json",
    yaml: "yaml",
    yml: "yaml",
    toml: "toml",
    md: "markdown",
    markdown: "markdown",
    sql: "sql",
    sh: "bash",
    bash: "bash",
    zsh: "bash",
    dockerfile: "dockerfile",
    xml: "xml",
    svg: "xml",
    ini: "ini",
    bat: "bat",
    cmd: "bat",
    ps1: "powershell",
    lua: "lua",
    php: "php",
    rb: "ruby",
    swift: "swift",
    kt: "kotlin",
    kts: "kotlin",
    dart: "dart",
  };
  return map[ext] || getLang(filePath) || "plaintext";
}

let currentLoadId = 0;
async function buildDiffFile() {
  const loadId = ++currentLoadId;
  const d = activeDiff.value;
  if (!d || isBinary.value) {
    currentDiffFile.value = null;
    return;
  }

  isBuildingDiff.value = true;
  isAllExpanded.value = false;
  try {
    const hl = await getHighlighter();
    if (loadId !== currentLoadId) return;

    const lang = resolveLang(d.path);
    const engine = hl.getHighlighterEngine?.();
    if (engine && !engine.getLoadedLanguages().includes(lang)) {
      try {
        await engine.loadLanguage(lang);
      } catch (err) {
        console.warn(`Could not load Shiki language "${lang}":`, err);
      }
    }

    const file = generateDiffFile(
      d.path,
      d.oldContent,
      d.path,
      d.newContent,
      lang,
      lang
    );
    file.initTheme("dark");
    file.initRaw();
    file.initSyntax({ registerHighlighter: hl });
    file.buildSplitDiffLines();
    file.buildUnifiedDiffLines();

    if (loadId !== currentLoadId) return;
    currentDiffFile.value = file;
  } catch (err) {
    console.error("Failed to build diff file:", err);
    if (loadId === currentLoadId) {
      currentDiffFile.value = null;
    }
  } finally {
    if (loadId === currentLoadId) {
      isBuildingDiff.value = false;
    }
  }
}

watch(activeDiff, () => {
  buildDiffFile();
});

async function loadDiff(hash: string) {
  const path = effectivePath.value;
  if (!path) return;
  loadingDiff.value = true;
  try {
    const ds = await api.gitShowCommit(path, hash);
    diffs.value = ds;
    activePath.value = ds[0]?.path ?? null;
  } catch (e: any) {
    errorMsg.value = e?.message ?? String(e);
  } finally {
    loadingDiff.value = false;
  }
}

function selectCommit(hash: string) {
  selectedHash.value = hash;
  loadDiff(hash);
}

async function load() {
  const path = effectivePath.value;
  if (!path) return;
  loading.value = true;
  errorMsg.value = "";
  try {
    commits.value = await api.gitLog(path, 200);
    selectedHash.value = null;
    diffs.value = [];
    activePath.value = null;
    currentDiffFile.value = null;
    if (commits.value.length > 0) {
      selectedHash.value = commits.value[0].hash;
      await loadDiff(commits.value[0].hash);
    }
  } catch (e: any) {
    errorMsg.value = e?.message ?? String(e);
  } finally {
    loading.value = false;
  }
}

watch(effectivePath, (path) => {
  if (path && path !== props.projectPath) emit("select", path);
  load();
});

onMounted(async () => {
  try {
    const hl = await getHighlighter();
    const engine = hl.getHighlighterEngine?.();
    if (engine) {
      await Promise.allSettled([
        engine.loadLanguage("rust"),
        engine.loadLanguage("toml"),
        engine.loadLanguage("yaml"),
      ]);
    }
  } catch (err) {
    console.warn("Warm up highlighter failed:", err);
  }
  if (effectivePath.value && effectivePath.value !== props.projectPath) emit("select", effectivePath.value);
  load();
});
</script>

<template>
  <div class="flex flex-col flex-1 min-h-0 h-full p-4 gap-3">
    <div class="flex items-center gap-3 shrink-0">
      <span class="text-[13px] text-foreground/40 uppercase tracking-wider">{{ t("historyTitle") }}</span>
      <span v-if="effectiveName" class="text-[13px] text-primary font-mono truncate max-w-[420px]">{{ effectiveName }}</span>
      <div class="flex-1" />
      <Button variant="outline" size="sm" :disabled="!effectivePath || loading" @click="load">
        {{ loading ? t("loading") : t("reload") }}
      </Button>
    </div>

    <div v-if="errorMsg" class="shrink-0 text-[12px] text-destructive">{{ errorMsg }}</div>

    <div v-if="props.projects.length === 0" class="flex-1 flex items-center justify-center text-foreground/25 text-sm">
      {{ t("noProjectsHint") }}
    </div>

    <div v-else-if="loading" class="flex-1 flex items-center justify-center text-foreground/20 text-sm">
      {{ t("loadingHistory") }}
    </div>

    <div v-else-if="commits.length === 0" class="flex-1 flex items-center justify-center text-foreground/25 text-sm">
      {{ t("noCommits") }}
    </div>

    <div v-else class="flex flex-1 min-h-0 gap-3">
      <!-- Commits List Column -->
      <div class="w-[320px] shrink-0 rounded-xl border border-border/40 flex flex-col min-h-0 overflow-hidden bg-background/40">
        <div class="px-3 py-2 text-[11px] text-foreground/40 uppercase tracking-wider border-b border-border/30 shrink-0">
          {{ t("commitsCount", { count: commits.length }) }}
        </div>
        <div v-if="loadingDiff" class="px-3 py-2 text-[11px] text-foreground/40">{{ t("loadingDiff") }}</div>
        <div class="flex-1 overflow-y-auto">
          <div
            v-for="c in commits"
            :key="c.hash"
            class="group w-full text-left px-3 py-2.5 border-b border-border/10 hover:bg-white/[0.03] transition-colors cursor-pointer relative"
            :class="c.hash === selectedHash ? 'bg-primary/10 border-l-2 border-l-primary' : ''"
            @click="selectCommit(c.hash)"
          >
            <div class="flex items-start justify-between gap-1.5">
              <div class="text-[12px] text-white/90 truncate flex-1 font-medium leading-snug" :title="c.subject">
                {{ c.subject }}
              </div>
              <!-- Action buttons on hover -->
              <div class="opacity-0 group-hover:opacity-100 flex items-center gap-0.5 shrink-0 transition-opacity -mr-1 -mt-0.5 bg-[#0b161b]/95 px-1 py-0.5 rounded border border-border/40 shadow-sm">
                <!-- Copy message -->
                <button
                  type="button"
                  class="p-1 rounded hover:bg-white/10 text-white/40 hover:text-white/90 transition-colors"
                  :title="copiedType === c.hash + '-msg' ? t('copiedCommitMsg') : t('copyCommitMsg')"
                  @click.stop="copyText(c.subject, c.hash + '-msg')"
                >
                  <svg v-if="copiedType !== c.hash + '-msg'" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
                  </svg>
                  <svg v-else class="w-3 h-3 text-emerald-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                </button>
                <!-- Copy ID (hash) -->
                <button
                  type="button"
                  class="p-1 rounded hover:bg-white/10 text-white/40 hover:text-white/90 transition-colors"
                  :title="copiedType === c.hash + '-id' ? t('copiedCommitId') : t('copyCommitId')"
                  @click.stop="copyText(c.hash, c.hash + '-id')"
                >
                  <svg v-if="copiedType !== c.hash + '-id'" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                  </svg>
                  <svg v-else class="w-3 h-3 text-emerald-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                </button>
              </div>
            </div>

            <div class="text-[11px] text-white/35 mt-1 flex items-center justify-between gap-2">
              <button
                type="button"
                class="font-mono text-primary/80 hover:text-primary hover:underline shrink-0 flex items-center gap-1 group/btn"
                :title="copiedType === c.hash + '-id' ? t('copiedCommitId') : t('copyCommitId')"
                @click.stop="copyText(c.hash, c.hash + '-id')"
              >
                <span>{{ c.shortHash }}</span>
                <span v-if="copiedType === c.hash + '-id'" class="text-[10px] text-emerald-400 font-sans">✓ {{ t("copiedCommitId") }}</span>
                <svg v-else class="w-2.5 h-2.5 opacity-40 group-hover/btn:opacity-100 transition-opacity" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                </svg>
              </button>
              <span class="truncate">{{ c.author }}</span>
            </div>

            <div class="text-[10px] text-white/25 mt-0.5">{{ fmtDate(c.date) }}</div>
          </div>
        </div>
      </div>

      <!-- Diff Viewer Column -->
      <div class="flex-1 min-h-0 rounded-xl border border-border/40 flex flex-col overflow-hidden bg-[#0d1117]">
        <div v-if="!selectedCommit" class="flex-1 flex items-center justify-center text-foreground/25 text-sm">
          {{ t("selectCommitPrompt") }}
        </div>
        <template v-else>
          <!-- Selected Commit Summary Header -->
          <div class="px-4 py-2.5 bg-[#121c22] border-b border-border/30 shrink-0 flex items-center justify-between gap-3">
            <div class="min-w-0 flex-1">
              <div class="text-[13px] font-semibold text-white/95 truncate" :title="selectedCommit.subject">
                {{ selectedCommit.subject }}
              </div>
              <div class="flex items-center gap-2.5 text-[11px] text-white/40 mt-0.5 font-mono">
                <span class="text-primary/90 font-bold">{{ selectedCommit.shortHash }}</span>
                <span>•</span>
                <span class="text-white/60">{{ selectedCommit.author }}</span>
                <span>•</span>
                <span>{{ fmtDate(selectedCommit.date) }}</span>
              </div>
            </div>

            <!-- Action buttons -->
            <div class="flex items-center gap-1.5 shrink-0">
              <button
                type="button"
                class="px-2.5 py-1 rounded-md bg-white/[0.04] hover:bg-white/[0.08] text-white/70 hover:text-white text-[11px] font-mono flex items-center gap-1.5 transition-colors border border-border/30"
                :title="t('copyCommitId')"
                @click="copyText(selectedCommit.hash, 'header-id')"
              >
                <svg v-if="copiedType !== 'header-id'" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                </svg>
                <svg v-else class="w-3 h-3 text-emerald-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                  <polyline points="20 6 9 17 4 12" />
                </svg>
                <span>{{ copiedType === 'header-id' ? t('copiedCommitId') : 'Copy ID' }}</span>
              </button>

              <button
                type="button"
                class="px-2.5 py-1 rounded-md bg-white/[0.04] hover:bg-white/[0.08] text-white/70 hover:text-white text-[11px] flex items-center gap-1.5 transition-colors border border-border/30"
                :title="t('copyCommitMsg')"
                @click="copyText(selectedCommit.subject, 'header-msg')"
              >
                <svg v-if="copiedType !== 'header-msg'" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
                </svg>
                <svg v-else class="w-3 h-3 text-emerald-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                  <polyline points="20 6 9 17 4 12" />
                </svg>
                <span>{{ copiedType === 'header-msg' ? t('copiedCommitMsg') : 'Copy Message' }}</span>
              </button>
            </div>
          </div>

          <!-- Changed Files Tabs -->
          <div class="px-3 py-1.5 border-b border-border/30 shrink-0 flex items-center gap-2 overflow-x-auto bg-[#0a1216]/70">
            <span class="text-[10px] text-foreground/40 uppercase tracking-wider shrink-0">{{ t("files") }}</span>
            <button
              v-for="d in diffs"
              :key="d.path"
              class="px-2 py-1 rounded-md text-[11px] font-mono whitespace-nowrap transition-colors flex items-center gap-1.5"
              :class="d.path === activePath ? 'bg-primary/15 text-primary' : 'text-white/50 hover:bg-white/[0.04] hover:text-white/80'"
              @click="activePath = d.path"
            >
              <span :class="statusTint(d.status)" class="font-bold text-[10px]">{{ d.status }}</span>
              <span>{{ d.path }}</span>
            </button>
            <span v-if="activeDiff" class="flex-1" />
            <span v-if="activeDiff" class="text-[11px] font-mono text-white/25 shrink-0">
              {{ statusLabel[activeDiff.status] ?? activeDiff.status }}
            </span>
          </div>

          <div v-if="!activeDiff" class="flex-1 flex items-center justify-center text-foreground/25 text-sm">
            {{ t("noFileChanges") }}
          </div>

          <template v-else>
            <!-- Diff File Header matching screenshot -->
            <div class="flex items-center justify-between px-3 py-2 bg-[#131b22] border-b border-border/30 select-none text-[12px] shrink-0">
              <!-- Left: chevron + file icon + path + copy button -->
              <div class="flex items-center gap-2 min-w-0">
                <button
                  class="p-1 rounded hover:bg-white/[0.08] text-white/60 hover:text-white/90 transition-colors"
                  :title="isCollapsed ? 'Mở rộng' : 'Thu gọn'"
                  @click="isCollapsed = !isCollapsed"
                >
                  <svg
                    class="w-3.5 h-3.5 transition-transform duration-150"
                    :class="isCollapsed ? '-rotate-90' : ''"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <polyline points="6 9 12 15 18 9" />
                  </svg>
                </button>

                <!-- File Icon -->
                <svg class="w-3.5 h-3.5 text-white/50 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                  <polyline points="14 2 14 8 20 8" />
                </svg>

                <!-- Path -->
                <span class="font-mono text-white/90 font-medium truncate" :title="activeDiff.path">{{ activeDiff.path }}</span>

                <!-- Copy button -->
                <button
                  class="p-1 rounded hover:bg-white/[0.08] text-white/40 hover:text-white/80 transition-colors"
                  :title="copied ? 'Đã sao chép' : 'Sao chép đường dẫn'"
                  @click="copyPath(activeDiff.path)"
                >
                  <svg v-if="!copied" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                  </svg>
                  <svg v-else class="w-3.5 h-3.5 text-emerald-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                </button>
              </div>

              <!-- Right: Stats + Split/Unified Toggle + Wrap + Expand all -->
              <div class="flex items-center gap-3 shrink-0 text-white/60">
                <!-- Addition / Deletion statistics -->
                <div v-if="currentDiffFile" class="flex items-center gap-2 font-mono text-[11px] font-semibold">
                  <span class="text-emerald-400">+{{ currentDiffFile.additionLength }}</span>
                  <span class="text-rose-400">-{{ currentDiffFile.deletionLength }}</span>
                </div>

                <div class="h-3.5 w-px bg-white/10" />

                <!-- Split / Unified Toggle -->
                <div class="flex items-center rounded-md bg-white/[0.04] p-0.5 border border-white/10 text-[11px]">
                  <button
                    class="px-2 py-0.5 rounded transition-colors flex items-center gap-1 font-mono"
                    :class="diffMode === DiffModeEnum.SplitGitHub ? 'bg-primary/25 text-primary font-medium' : 'text-white/50 hover:text-white/90'"
                    title="Split (Side-by-side)"
                    @click="diffMode = DiffModeEnum.SplitGitHub"
                  >
                    <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="12" y1="3" x2="12" y2="21"/></svg>
                    <span>Split</span>
                  </button>
                  <button
                    class="px-2 py-0.5 rounded transition-colors flex items-center gap-1 font-mono"
                    :class="diffMode === DiffModeEnum.Unified ? 'bg-primary/25 text-primary font-medium' : 'text-white/50 hover:text-white/90'"
                    title="Unified (Inline)"
                    @click="diffMode = DiffModeEnum.Unified"
                  >
                    <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="3" y1="15" x2="21" y2="15"/></svg>
                    <span>Unified</span>
                  </button>
                </div>

                <!-- Wrap Lines Toggle -->
                <button
                  class="p-1 rounded hover:bg-white/[0.08] transition-colors"
                  :class="wrapLines ? 'text-primary' : 'text-white/40 hover:text-white/80'"
                  :title="wrapLines ? 'Tắt tự động xuống dòng' : 'Bật tự động xuống dòng'"
                  @click="wrapLines = !wrapLines"
                >
                  <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="3" y1="6" x2="21" y2="6" /><path d="M3 12h15a3 3 0 1 1 0 6h-4" /><polyline points="16 16 14 18 16 20" /><line x1="3" y1="18" x2="10" y2="18" />
                  </svg>
                </button>

                <!-- Expand / Collapse All Lines -->
                <button
                  v-if="currentDiffFile"
                  class="p-1 rounded hover:bg-white/[0.08] text-white/40 hover:text-white/80 transition-colors"
                  :title="isAllExpanded ? 'Thu gọn các đoạn code không đổi' : 'Mở rộng tất cả dòng code'"
                  @click="toggleExpandAll"
                >
                  <svg v-if="!isAllExpanded" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="15 3 21 3 21 9" /><polyline points="9 21 3 21 3 15" /><line x1="21" y1="3" x2="14" y2="10" /><line x1="3" y1="21" x2="10" y2="14" />
                  </svg>
                  <svg v-else class="w-3.5 h-3.5 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="4 14 10 14 10 20" /><polyline points="20 10 14 10 14 4" /><line x1="14" y1="10" x2="21" y2="3" /><line x1="3" y1="21" x2="10" y2="14" />
                  </svg>
                </button>
              </div>
            </div>

            <!-- Diff Content View -->
            <div v-show="!isCollapsed" class="flex-1 min-h-0 relative overflow-hidden bg-[#0d1117]">
              <!-- Loading Overlay -->
              <div v-if="isBuildingDiff" class="absolute inset-0 flex items-center justify-center bg-[#0d1117]/80 z-20 gap-2 text-white/60 text-sm">
                <svg class="animate-spin h-5 w-5 text-primary" viewBox="0 0 24 24" fill="none">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
                </svg>
                <span>Đang phân tích cú pháp & so khớp code...</span>
              </div>

              <!-- Binary File Warning -->
              <div v-if="isBinary" class="flex-1 h-full flex flex-col items-center justify-center text-white/40 gap-2 p-8">
                <svg class="w-10 h-10 text-white/20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                  <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18"/>
                  <line x1="7" y1="2" x2="7" y2="22"/>
                  <line x1="17" y1="2" x2="17" y2="22"/>
                  <line x1="2" y1="12" x2="22" y2="12"/>
                </svg>
                <div class="text-[13px] font-mono text-center">File nhị phân không hỗ trợ xem khác biệt nội dung</div>
              </div>

              <!-- Empty Diff -->
              <div v-else-if="!currentDiffFile && !isBuildingDiff" class="flex-1 h-full flex items-center justify-center text-white/30 text-sm">
                {{ t("noFileChanges") }}
              </div>

              <!-- DiffView Component -->
              <div v-else class="h-full w-full overflow-auto diff-view-scroll-container">
                <DiffView
                  :diff-file="currentDiffFile!"
                  :diff-view-mode="diffMode"
                  diff-view-theme="dark"
                  :diff-view-wrap="wrapLines"
                  :diff-view-highlight="true"
                  :register-highlighter="highlighterInstance"
                  class="text-[12px] font-mono"
                />
              </div>
            </div>
          </template>
        </template>
      </div>
    </div>
  </div>
</template>

<style>
.diff-view-scroll-container .diff-tailwindcss-wrapper {
  background-color: #0d1117 !important;
  min-height: 100%;
}
.diff-view-scroll-container table {
  border-collapse: collapse;
}
.diff-view-scroll-container pre {
  font-family: var(--font-mono, monospace) !important;
}

.diff-view-scroll-container [data-theme="dark"] .diff-line-syntax-raw span[style*="--diff-view-dark"] {
  color: var(--diff-view-dark) !important;
  font-weight: var(--diff-view-dark-font-weight, inherit);
  text-decoration: var(--diff-view-dark-text-decoration, inherit);
}

.diff-view-scroll-container [data-theme="light"] .diff-line-syntax-raw span[style*="--diff-view-light"] {
  color: var(--diff-view-light) !important;
  font-weight: var(--diff-view-light-font-weight, inherit);
  text-decoration: var(--diff-view-light-text-decoration, inherit);
}
</style>