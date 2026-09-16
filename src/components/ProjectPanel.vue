<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import Button from "@/components/ui/Button.vue";
import Badge from "@/components/ui/Badge.vue";
import ScrollArea from "@/components/ui/ScrollArea.vue";
import * as api from "@/api/tauri";
import type { Provider, RepoStatus } from "@/lib/types";
import { t } from "@/lib/i18n";

const props = defineProps<{
  path: string;
  name: string;
}>();

const emit = defineEmits<{
  committed: [] | [err: string];
  openHistory: [path: string];
  removed: [path: string];
}>();

const status = ref<RepoStatus | null>(null);
const providers = ref<Provider[]>([]);
const providerId = ref("");
const note = ref("");
const message = ref("");
const generating = ref(false);
const committing = ref(false);
const initializing = ref(false);
const busy = ref(false);
const errorMsg = ref("");
const infoMsg = ref("");
const msgEl = ref<HTMLTextAreaElement | null>(null);

const isNotGitRepo = computed(() => {
  const err = (errorMsg.value || "").toLowerCase();
  return err.includes("not a git repository") || err.includes("không phải là thư mục git");
});

const providerName = computed(() => providers.value.find((p) => p.id === providerId.value)?.name ?? t("noProviderSelected"));
const hasChanges = computed(() => status.value !== null && !status.value.clean);
const canGenerate = computed(() => !generating.value && !committing.value && Boolean(providerId.value) && hasChanges.value);
const canCommit = computed(() => !generating.value && !committing.value && message.value.trim().length > 0);

async function loadStatus() {
  try {
    status.value = await api.gitStatus(props.path);
    errorMsg.value = "";
  } catch (e: any) {
    status.value = null;
    errorMsg.value = e?.message || String(e);
  }
}

async function loadProviders() {
  providers.value = await api.listProviders();
  if (!providerId.value && providers.value.length) {
    providerId.value = providers.value[0].id;
  }
}

async function doInit() {
  initializing.value = true;
  errorMsg.value = "";
  try {
    await api.gitInit(props.path);
    infoMsg.value = t("initSuccess");
    await loadStatus();
    emit("committed");
  } catch (e: any) {
    errorMsg.value = e?.message || String(e);
  } finally {
    initializing.value = false;
  }
}

async function removeCurrentProject() {
  try {
    await api.removeProject(props.path);
    emit("removed", props.path);
  } catch (e: any) {
    errorMsg.value = e?.message || String(e);
  }
}

function scrollMsgToBottom() {
  requestAnimationFrame(() => {
    if (msgEl.value) msgEl.value.scrollTop = msgEl.value.scrollHeight;
  });
}

async function generate() {
  generating.value = true;
  errorMsg.value = "";
  infoMsg.value = "";
  message.value = "";
  try {
    await api.streamCommitMessage(props.path, providerId.value, note.value, (e) => {
      if (e.type === "chunk") {
        message.value += e.text;
        scrollMsgToBottom();
      } else if (e.type === "done") {
        message.value = e.message;
        infoMsg.value = t("aiDone");
      } else if (e.type === "error") {
        errorMsg.value = e.message;
      }
    });
  } catch (e: any) {
    errorMsg.value = e?.message || String(e);
  } finally {
    generating.value = false;
  }
}

async function commit() {
  committing.value = true;
  errorMsg.value = "";
  infoMsg.value = "";
  try {
    const result = await api.gitCommit(props.path, message.value);
    infoMsg.value = t("committed", { hash: result.shortHash });
    message.value = "";
    note.value = "";
    await loadStatus();
    emit("committed");
  } catch (e: any) {
    errorMsg.value = e?.message || String(e);
    emit("committed", e?.message || String(e));
  } finally {
    committing.value = false;
  }
}

onMounted(async () => {
  busy.value = true;
  await Promise.all([loadStatus(), loadProviders()]);
  busy.value = false;
});

watch(
  () => props.path,
  () => {
    message.value = "";
    note.value = "";
    errorMsg.value = "";
    infoMsg.value = "";
    busy.value = true;
    Promise.all([loadStatus(), loadProviders()]).finally(() => (busy.value = false));
  },
);

function fileGlyph(f: { index: string; worktree: string }) {
  if (f.index === "?" || f.worktree === "?") return { c: "text-cyan-400", g: "?" };
  if (f.index !== " " && f.index !== "U") return { c: "text-emerald-400", g: "A" };
  if (f.index === "U" || f.worktree === "U") return { c: "text-rose-400", g: "U" };
  if (f.worktree !== " ") return { c: "text-amber-400", g: "M" };
  return { c: "text-slate-400", g: f.index };
}
</script>

<template>
  <div class="flex flex-col flex-1 min-h-0 bg-card/40 rounded-2xl border border-border/40 mx-3 my-2 overflow-hidden shadow-2xl">
    <!-- header -->
    <div class="flex items-center gap-3 px-5 py-4 border-b border-border/30 shrink-0">
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <span class="text-[15px] font-semibold text-foreground">{{ name }}</span>
          <Badge v-if="!isNotGitRepo" variant="secondary">{{ status?.branch || "…" }}</Badge>
          <Badge v-else variant="destructive">{{ t("noGitBadge") }}</Badge>
          <span v-if="status?.ahead_behind" class="text-[11px] text-foreground/30 font-mono">{{ status.ahead_behind }}</span>
        </div>
        <div class="text-[11px] text-foreground/30 font-mono truncate mt-0.5">{{ path }}</div>
      </div>
      <div v-if="status" class="flex items-center gap-2 shrink-0">
        <Badge :variant="hasChanges ? 'destructive' : 'secondary'">
          {{ status.clean ? t("cleanRepo") : t("statusLine", { staged: status.staged, unstaged: status.unstaged, untracked: status.untracked }) }}
        </Badge>
        <Button variant="ghost" size="sm" @click="emit('openHistory', path)">{{ t("openHistory") }}</Button>
      </div>
      <div v-else-if="isNotGitRepo" class="flex items-center gap-2 shrink-0">
        <Button variant="outline" size="sm" class="text-destructive hover:bg-destructive/10" @click="removeCurrentProject">
          {{ t("removeProject") }}
        </Button>
        <Button variant="default" size="sm" :disabled="initializing" @click="doInit">
          {{ initializing ? t("initializing") : t("gitInit") }}
        </Button>
      </div>
    </div>

    <div v-if="busy" class="flex-1 flex items-center justify-center text-foreground/20 text-sm">{{ t("loading") }}</div>

    <!-- uninitialized git repo state -->
    <div v-else-if="isNotGitRepo" class="flex-1 flex flex-col items-center justify-center p-8 text-center">
      <div class="w-16 h-16 rounded-2xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center mb-4 text-amber-400">
        <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
      </div>
      <h3 class="text-[17px] font-semibold text-foreground mb-1.5">{{ t("notAGitRepo") }}</h3>
      <p class="text-[13px] text-foreground/50 max-w-md mb-6 leading-relaxed">
        {{ t("notAGitRepoDesc") }}
      </p>
      <div class="flex items-center gap-3">
        <Button variant="default" size="default" :disabled="initializing" class="gap-2 font-medium" @click="doInit">
          <svg v-if="initializing" class="animate-spin h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
          </svg>
          {{ initializing ? t("initializing") : t("initGitButton") }}
        </Button>
        <Button variant="ghost" size="default" @click="removeCurrentProject">
          {{ t("removeProject") }}
        </Button>
      </div>
      <div v-if="errorMsg && !initializing" class="mt-4 text-[12px] font-mono text-destructive/80 bg-destructive/10 px-3 py-1.5 rounded-lg max-w-lg truncate">
        {{ errorMsg }}
      </div>
    </div>

    <template v-else>
      <!-- status / changed files -->
      <ScrollArea class="shrink-0 max-h-[160px] px-5 py-3 border-b border-border/20">
        <p v-if="status && status.clean" class="text-[12px] text-foreground/40">{{ t("noChanges") }}</p>
        <div v-else class="flex flex-col gap-1">
          <button
            v-for="(f, i) in status?.files"
            :key="i"
            type="button"
            class="flex items-center gap-2 text-left text-[12px] font-mono px-2 py-1 rounded-md hover:bg-white/[0.04]"
          >
            <span :class="fileGlyph(f).c" class="w-5 shrink-0">{{ fileGlyph(f).g }}</span>
            <span class="text-foreground/60 truncate">{{ f.path }}</span>
          </button>
        </div>
      </ScrollArea>

      <!-- AI controls -->
      <div class="flex items-center gap-3 px-5 py-3 border-b border-border/20 shrink-0">
        <label class="text-[11px] text-foreground/40 uppercase tracking-wider shrink-0">{{ t("aiProvider") }}</label>
        <select
          v-model="providerId"
          class="bg-white/[0.04] border border-border/40 rounded-lg px-2 py-1.5 text-[12px] text-foreground outline-none focus:border-primary/50 min-w-[180px]"
        >
          <option v-if="providers.length === 0" value="" disabled>{{ t("noProviderOption") }}</option>
          <option v-for="p in providers" :key="p.id" :value="p.id">{{ p.name }} · {{ p.model }}</option>
        </select>
        <span v-if="providerName && !canGenerate" class="text-[11px] text-foreground/25 shrink-0">{{ providerName }}</span>
        <Button
          variant="default"
          size="sm"
          class="whitespace-nowrap"
          :disabled="!canGenerate"
          @click="generate"
        >
          {{ generating ? t("aiGenerating") : t("generateWithAi") }}
        </Button>
      </div>

      <!-- note + message editor (fits the frame) -->
      <div class="flex flex-col flex-1 min-h-0 gap-2 px-5 py-3">
        <input
          v-model="note"
          type="text"
          :placeholder="t('notePlaceholder')"
          class="bg-white/[0.04] border border-border/30 rounded-lg px-3 py-1.5 text-[12px] text-foreground placeholder-foreground/25 outline-none focus:border-primary/40 shrink-0"
        />
        <textarea
          ref="msgEl"
          v-model="message"
          :placeholder="t('messagePlaceholder')"
          class="flex-1 w-full min-h-0 resize-none bg-[#0c1c22] border border-border/30 rounded-lg px-3 py-2 text-[13px] font-mono text-foreground leading-relaxed outline-none focus:border-primary/50 placeholder-foreground/25"
        ></textarea>
      </div>

      <!-- footer -->
      <div class="flex items-center gap-3 px-5 py-3 border-t border-border/30 shrink-0">
        <p v-if="errorMsg" class="flex-1 text-[12px] text-destructive truncate">{{ errorMsg }}</p>
        <p v-else-if="infoMsg" class="flex-1 text-[12px] text-primary truncate">{{ infoMsg }}</p>
        <p v-else class="flex-1 text-[12px] text-foreground/25 truncate">{{ t("footerHint") }}</p>
        <Button variant="outline" size="sm" :disabled="busy" @click="loadStatus">{{ t("refresh") }}</Button>
        <Button variant="default" size="sm" class="whitespace-nowrap" :disabled="!canCommit" @click="commit">
          {{ committing ? t("committing") : t("commit") }}
        </Button>
      </div>
    </template>
  </div>
</template>