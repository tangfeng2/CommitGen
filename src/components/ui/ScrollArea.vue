<script setup lang="ts">
import { computed, useAttrs } from "vue";
import { cn } from "@/lib/utils";

defineOptions({ inheritAttrs: false });

const attrs = useAttrs();
const rootClass = computed(() => cn("relative overflow-auto", attrs.class as string | undefined));
</script>

<template>
  <div v-bind="{ ...attrs, class: undefined }" data-slot="scroll-area" :class="rootClass">
    <slot />
  </div>
</template>