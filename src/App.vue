<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import Button from "@/components/ui/Button.vue";
import ProjectList from "@/components/ProjectList.vue";
import ProjectPanel from "@/components/ProjectPanel.vue";
import HistoryTab from "@/components/HistoryTab.vue";
import ProvidersTab from "@/components/ProvidersTab.vue";
import * as api from "@/api/tauri";
import type { ProjectInfo, TabId } from "@/lib/types";
import { i18n, setLocale, t } from "@/lib/i18n";

const activeTab = ref<TabId>("projects");
const projects = ref<ProjectInfo[]>([]);
const selectedPath = ref<string | null>(null);

const selectedProject = () => projects.value.find((p) => p.path === selectedPath.value) ?? null;

async function refreshProjects() {
  projects.value = await api.listProjects();
}

function selectProject(path: string) {
  selectedPath.value = path;
  activeTab.value = "projects";
}

function openHistory(path: string) {
  selectedPath.value = path;
  activeTab.value = "history";
}

function handleProjectRemoved(path: string) {
  if (selectedPath.value === path) {
    selectedPath.value = null;
  }
  refreshProjects();
}

onMounted(refreshProjects);

const tabs = computed(() => [
  { id: "projects" as TabId, label: t("tabProjects") },
  { id: "history" as TabId, label: t("tabHistory") },
  { id: "providers" as TabId, label: t("tabProviders") },
]);
</script>

<template>
  <div class="h-screen flex flex-col bg-background text-foreground">
    <header class="flex items-center justify-between px-5 h-12 border-b border-border bg-[#0b1920]/80 backdrop-blur-md shrink-0">
      <div class="flex items-center gap-6">
        <div class="flex items-center gap-2.5">
          <div class="w-[7px] h-[7px] rounded-full bg-primary" />
          <span class="text-[13px] font-semibold text-foreground/90 tracking-tight">CommitGen</span>
        </div>

        <nav class="flex items-center gap-0.5">
          <Button
            v-for="tab in tabs"
            :key="tab.id"
            variant="ghost"
            size="xs"
            class="px-3 py-1 rounded-md text-[12px] font-medium transition-all"
            :class="activeTab === tab.id ? 'bg-primary/15 text-primary' : 'text-muted-foreground hover:text-foreground hover:bg-white/[0.04]'"
            @click="activeTab = tab.id"
          >
            {{ tab.label }}
          </Button>
        </nav>
      </div>

      <div class="flex items-center gap-2">
        <Button variant="ghost" size="xs" class="px-2 py-1 rounded-md text-[11px] text-muted-foreground hover:text-foreground" @click="setLocale(i18n.locale === 'en' ? 'vi' : 'en')">
          {{ t("switchLocale") }}
        </Button>
        <div
          v-if="selectedProject()"
          class="flex items-center gap-2 px-3 py-1 rounded-lg bg-white/[0.04] border border-border/30"
        >
          <span class="text-[10px] text-foreground/30 uppercase tracking-wider">{{ t("selectedProject") }}</span>
          <span class="text-[11px] text-primary font-mono truncate max-w-[280px]">{{ selectedProject()?.path }}</span>
        </div>
      </div>
    </header>

    <main class="flex-1 min-h-0">
      <template v-if="activeTab === 'projects'">
        <div class="flex flex-1 min-h-0 h-full">
          <div class="w-[300px] shrink-0 border-r border-border/30 flex flex-col min-h-0">
            <ProjectList
              :projects="projects"
              :selected-path="selectedPath"
              :refresh="refreshProjects"
              :select-project="selectProject"
            />
          </div>
          <template v-if="selectedPath && selectedProject()">
            <ProjectPanel
              :path="selectedPath"
              :name="selectedProject()!.name"
              @open-history="openHistory"
              @committed="refreshProjects"
              @removed="handleProjectRemoved"
            />
          </template>
          <div v-else class="flex-1 flex items-center justify-center text-foreground/25 text-sm">
            Click vào một dự án để thả cửa sổ tạo commit với AI.
          </div>
        </div>
      </template>
      <HistoryTab
        v-else-if="activeTab === 'history'"
        :project-path="selectedPath"
        :projects="projects"
        @select="(p) => (selectedPath = p)"
      />
      <ProvidersTab v-else />
    </main>
  </div>
</template>