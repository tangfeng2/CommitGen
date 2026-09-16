<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import Button from "@/components/ui/Button.vue";
import Input from "@/components/ui/Input.vue";
import Badge from "@/components/ui/Badge.vue";
import ScrollArea from "@/components/ui/ScrollArea.vue";
import * as api from "@/api/tauri";
import type { Provider } from "@/lib/types";
import { t } from "@/lib/i18n";

const providers = ref<Provider[]>([]);
const editing = ref<Provider | null>(null);
const form = reactive({ name: "", base_url: "", api_key: "", model: "", system_prompt: "" });
const saving = ref(false);
const msg = ref("");
const err = ref("");

const formTitle = computed(() => (editing.value ? t("editProviderTitle", { name: editing.value.name }) : t("addProviderTitle")));

const presets: { label: string; base_url: string; model: string }[] = [
  { label: "OpenAI", base_url: "https://api.openai.com/v1", model: "gpt-4o-mini" },
  { label: "OpenRouter", base_url: "https://openrouter.ai/api/v1", model: "openai/gpt-4o-mini" },
  { label: "DeepSeek", base_url: "https://api.deepseek.com/v1", model: "deepseek-chat" },
  { label: "Groq", base_url: "https://api.groq.com/openai/v1", model: "llama-3.3-70b-versatile" },
  { label: "Ollama (local)", base_url: "http://localhost:11434/v1", model: "llama3.2" },
];

function applyPreset(p: { base_url: string; model: string }) {
  form.base_url = p.base_url;
  form.model = p.model;
}

function startEdit(p: Provider) {
  editing.value = p;
  form.name = p.name;
  form.base_url = p.base_url;
  form.api_key = p.api_key;
  form.model = p.model;
  form.system_prompt = p.system_prompt ?? "";
  msg.value = "";
  err.value = "";
}

function resetForm() {
  editing.value = null;
  form.name = "";
  form.base_url = "";
  form.api_key = "";
  form.model = "";
  form.system_prompt = "";
  msg.value = "";
  err.value = "";
}

async function load() {
  providers.value = await api.listProviders();
}

async function save() {
  if (!form.name.trim() || !form.base_url.trim() || !form.model.trim()) {
    err.value = t("providerValidationErr");
    return;
  }
  saving.value = true;
  err.value = "";
  msg.value = "";
  try {
    const provider: Provider = {
      id: editing.value?.id ?? `${form.name.trim().toLowerCase().replace(/\s+/g, "-")}-${Date.now()}`,
      name: form.name.trim(),
      base_url: form.base_url.trim(),
      api_key: form.api_key.trim(),
      model: form.model.trim(),
      system_prompt: form.system_prompt.trim() || null,
      extra_headers: {},
    };
    await api.saveProvider(provider);
    msg.value = t("providerSaved");
    resetForm();
    await load();
  } catch (e: any) {
    err.value = e?.message || String(e);
  } finally {
    saving.value = false;
  }
}

async function remove(p: Provider) {
  await api.deleteProvider(p.id);
  if (editing.value?.id === p.id) resetForm();
  await load();
}

onMounted(load);
</script>

<template>
  <div class="flex flex-1 min-h-0 gap-4 p-4">
    <ScrollArea class="w-[300px] shrink-0 rounded-xl border border-border/40 bg-card/40 p-3">
      <div v-if="providers.length === 0" class="text-[12px] text-foreground/30 p-4 text-center">
        {{ t("noProviders") }}
      </div>
      <div v-else class="flex flex-col gap-2">
        <div
          v-for="p in providers"
          :key="p.id"
          class="group rounded-lg border border-border/30 bg-white/[0.02] p-3 flex items-center gap-2"
        >
          <button type="button" class="flex-1 min-w-0 text-left" @click="startEdit(p)">
            <div class="text-[13px] font-medium text-foreground">{{ p.name }}</div>
            <div class="text-[11px] font-mono text-foreground/30 truncate mt-0.5">{{ p.base_url }}</div>
            <Badge variant="secondary" class="mt-1.5">{{ p.model }}</Badge>
          </button>
          <Button variant="ghost" size="icon-xs" class="text-foreground/30 hover:text-destructive" @click.stop="remove(p)">x</Button>
        </div>
      </div>
    </ScrollArea>

    <div class="flex-1 flex flex-col gap-3 min-w-0">
      <div class="flex items-center gap-2">
        <span class="text-2xl">{{ editing ? "✏️" : "＋" }}</span>
        <h2 class="text-[15px] font-semibold">{{ formTitle }}</h2>
      </div>

      <div class="flex flex-wrap gap-1.5">
        <Button v-for="p in presets" :key="p.label" variant="outline" size="xs" class="px-2.5 py-1" @click="applyPreset(p)">
          {{ p.label }}
        </Button>
      </div>

      <div class="flex flex-col gap-3 rounded-xl border border-border/40 bg-card/40 p-4">
        <div class="grid grid-cols-2 gap-3">
          <label class="flex flex-col gap-1">
            <span class="text-[11px] text-foreground/40 uppercase tracking-wider">{{ t("providerNameField") }}</span>
            <Input v-model="form.name" :placeholder="t('providerNamePlaceholder')" class="bg-white/[0.04] border-white/[0.06] rounded-lg" />
          </label>
          <label class="flex flex-col gap-1">
            <span class="text-[11px] text-foreground/40 uppercase tracking-wider">{{ t("modelField") }}</span>
            <Input v-model="form.model" :placeholder="t('modelPlaceholder')" class="bg-white/[0.04] border-white/[0.06] rounded-lg" />
          </label>
        </div>
        <label class="flex flex-col gap-1">
          <span class="text-[11px] text-foreground/40 uppercase tracking-wider">{{ t("baseUrlField") }}</span>
          <Input v-model="form.base_url" :placeholder="t('baseUrlPlaceholder')" class="bg-white/[0.04] border-white/[0.06] rounded-lg" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-[11px] text-foreground/40 uppercase tracking-wider">{{ t("apiKeyField") }}</span>
          <Input v-model="form.api_key" type="password" :placeholder="t('apiKeyPlaceholder')" class="bg-white/[0.04] border-white/[0.06] rounded-lg" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-[11px] text-foreground/40 uppercase tracking-wider">{{ t("systemPromptField") }}</span>
          <textarea
            v-model="form.system_prompt"
            rows="3"
            :placeholder="t('systemPromptPlaceholder')"
            class="bg-white/[0.04] border border-border/40 rounded-lg px-3 py-2 text-[12px] font-mono text-foreground placeholder-foreground/25 outline-none focus:border-primary/50 resize-none"
          ></textarea>
        </label>

        <p v-if="err" class="text-[12px] text-destructive">{{ err }}</p>
        <p v-else-if="msg" class="text-[12px] text-primary">{{ msg }}</p>

        <div class="flex gap-2">
          <Button variant="default" size="sm" :disabled="saving" @click="save">
            {{ saving ? t("saving") : editing ? t("saveChanges") : t("saveProvider") }}
          </Button>
          <Button v-if="editing" variant="ghost" size="sm" @click="resetForm">{{ t("cancel") }}</Button>
          <Button variant="ghost" size="sm" @click="load">{{ t("refresh") }}</Button>
        </div>
      </div>
    </div>
  </div>
</template>