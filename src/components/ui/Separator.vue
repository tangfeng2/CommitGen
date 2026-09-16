<script setup lang="ts">
import { computed, useAttrs } from "vue";
import { cn } from "@/lib/utils";

defineOptions({ inheritAttrs: false });

const props = withDefaults(
  defineProps<{
    orientation?: "horizontal" | "vertical";
    decorative?: boolean;
  }>(),
  {
    orientation: "horizontal",
    decorative: true,
  },
);

const attrs = useAttrs();
const separatorClass = computed(() =>
  cn(
    "shrink-0 bg-border",
    props.orientation === "horizontal" ? "h-px w-full" : "h-full w-px",
    attrs.class as string | undefined,
  ),
);
</script>

<template>
  <div
    v-bind="{ ...attrs, class: undefined }"
    data-slot="separator"
    :role="decorative ? 'none' : 'separator'"
    :aria-orientation="decorative ? undefined : orientation"
    :class="separatorClass"
  />
</template>