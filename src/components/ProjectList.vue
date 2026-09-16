<script setup lang="ts">
import { ref } from "vue";
import Button from "@/components/ui/Button.vue";
import Input from "@/components/ui/Input.vue";
import ScrollArea from "@/components/ui/ScrollArea.vue";
import * as api from "@/api/tauri";
import type { ProjectInfo } from "@/lib/types";
import { t } from "@/lib/i18n";

const props = defineProps<{
  projects: ProjectInfo[];
  selectedPath?: string | null;
  refresh: () => void;
  selectProject: (path: string) => void;
}>();

const pathInput = ref("");
const loading = ref(false);
const errorMsg = ref("");
const initPath = ref("");
const actionLoading = ref<string | null>(null);

async function tryAdd(p: string) {
  loading.value = true;
  errorMsg.value = "";
  initPath.value = "";
  try {
    const info = await api.addProject(p);
    pathInput.value = "";
    props.refresh();
    props.selectProject(info.path);
  } catch (e: any) {
    errorMsg.value = e?.message || String(e);
    initPath.value = p;
  } finally {
    loading.value = false;
  }
}

async function doInit(p?: string) {
  const target = p || initPath.value;
  if (!target) return;
  actionLoading.value = target;
  errorMsg.value = "";
  try {
    await api.gitInit(target);
    props.refresh();
    props.selectProject(target);
    initPath.value = "";
  } catch (e: any) {
    errorMsg.value = e?.message || String(e);
  } finally {
    actionLoading.value = null;
  }
}

async function removeProj(p: string) {
  actionLoading.value = p;
  try {
    await api.removeProject(p);
    props.refresh();
  } catch (e: any) {
    errorMsg.value = e?.message || String(e);
  } finally {
    actionLoading.value = null;
  }
}
</script>

<template>
  <div class="flex flex-col flex-1 min-h-0">
    <div class="px-4 py-3 border-b border-border/30 flex gap-2 items-center">
      <Input
        v-model="pathInput"
        type="text"
        :placeholder="t('addInputPlaceholder')"
        class="flex-1 h-auto bg-white/[0.04] border-white/[0.06] rounded-lg py-2 text-[13px] text-foreground placeholder-foreground/25 focus:border-primary/40"
        @keydown.enter="tryAdd(pathInput.trim())"
      />
      <Button variant="default" size="sm" class="whitespace-nowrap" :disabled="loading || !pathInput.trim()" @click="tryAdd(pathInput.trim())">
        {{ loading ? t("adding") : t("addProject") }}
      </Button>
    </div>

    <p v-if="errorMsg" class="px-4 py-2 text-[12px] text-destructive bg-destructive/10 flex items-center gap-2">
      <span class="flex-1">{{ errorMsg }}</span>
      <Button v-if="initPath" variant="default" size="xs" class="whitespace-nowrap" :disabled="loading" @click="doInit()">
        {{ t("gitInit") }}
      </Button>
    </p>

    <ScrollArea class="flex-1 min-h-0">
      <div v-if="projects.length === 0" class="flex items-center justify-center h-full text-foreground/20 text-[13px]">
        {{ t("noProjectsYet") }}
      </div>
      <div v-else class="flex flex-col gap-1 p-2">
        <div
          v-for="proj in projects"
          :key="proj.path"
          class="flex items-center gap-3 w-full text-left px-3.5 py-2.5 rounded-lg transition-colors group cursor-pointer"
          :class="selectedPath === proj.path ? 'bg-white/[0.08]' : 'hover:bg-white/[0.04]'"
          @click="selectProject(proj.path)"
        >
          <div
            class="w-[8px] h-[8px] rounded-full shrink-0"
            :class="proj.error ? 'bg-amber-400' : 'bg-primary'"
          />
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-1.5">
              <span class="text-[13px] text-foreground font-medium truncate">{{ proj.name }}</span>
              <span v-if="proj.error" class="text-[10px] text-amber-400 bg-amber-400/10 px-1.5 py-0.5 rounded font-mono shrink-0">
                {{ t("noGitBadge") }}
              </span>
            </div>
            <div class="text-[11px] text-foreground/30 font-mono truncate">{{ proj.path }}</div>
          </div>

          <!-- action buttons on hover -->
          <div class="opacity-0 group-hover:opacity-100 flex items-center gap-1 transition-opacity shrink-0">
            <Button
              v-if="proj.error"
              variant="outline"
              size="xs"
              class="h-6 px-2 text-[11px] text-amber-400 border-amber-400/30 hover:bg-amber-400/10"
              :disabled="actionLoading === proj.path"
              @click.stop="doInit(proj.path)"
            >
              {{ actionLoading === proj.path ? '...' : t("gitInit") }}
            </Button>
            <button
              type="button"
              class="p-1.5 rounded text-foreground/30 hover:text-destructive hover:bg-destructive/10 transition-colors"
              :title="t('removeProject')"
              :disabled="actionLoading === proj.path"
              @click.stop="removeProj(proj.path)"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </ScrollArea>
  </div>
</template>