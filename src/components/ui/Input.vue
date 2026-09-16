<script setup lang="ts">
import { computed, useAttrs } from "vue";
import { cn } from "@/lib/utils";

defineOptions({ inheritAttrs: false });

const model = defineModel<string | number>();
const attrs = useAttrs();
const inputClass = computed(() =>
  cn(
    "h-9 w-full min-w-0 rounded-md border border-input bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none selection:bg-primary selection:text-primary-foreground placeholder:text-muted-foreground disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm dark:bg-input/30",
    "focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50",
    attrs.class as string | undefined,
  ),
);
</script>

<template>
  <input v-bind="{ ...attrs, class: undefined }" v-model="model" data-slot="input" :class="inputClass" />
</template>