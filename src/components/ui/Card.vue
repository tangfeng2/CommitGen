<script setup lang="ts">
import { computed, useAttrs } from "vue";
import { cn } from "@/lib/utils";

defineOptions({ inheritAttrs: false });

const attrs = useAttrs();
const cardClass = computed(() =>
  cn("flex flex-col gap-6 rounded-xl border bg-card py-6 text-card-foreground shadow-sm", attrs.class as string | undefined),
);
</script>

<template>
  <div v-bind="{ ...attrs, class: undefined }" data-slot="card" :class="cardClass">
    <slot />
  </div>
</template>