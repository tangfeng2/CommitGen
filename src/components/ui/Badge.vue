<script setup lang="ts">
import { computed, useAttrs } from "vue";
import { cn } from "@/lib/utils";

defineOptions({ inheritAttrs: false });

const props = withDefaults(
  defineProps<{
    variant?: "default" | "secondary" | "destructive" | "outline" | "ghost" | "link";
  }>(),
  { variant: "default" },
);

const attrs = useAttrs();

const variantClasses = {
  default: "bg-primary text-primary-foreground",
  secondary: "bg-secondary text-secondary-foreground",
  destructive: "bg-destructive text-white",
  outline: "border-border text-foreground",
  ghost: "",
  link: "text-primary underline-offset-4 hover:underline",
};

const badgeClass = computed(() =>
  cn(
    "inline-flex w-fit shrink-0 items-center justify-center gap-1 overflow-hidden rounded-full border border-transparent px-2 py-0.5 text-xs font-medium whitespace-nowrap transition-[color,box-shadow] focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50",
    variantClasses[props.variant],
    attrs.class as string | undefined,
  ),
);
</script>

<template>
  <span v-bind="{ ...attrs, class: undefined }" data-slot="badge" :class="badgeClass">
    <slot />
  </span>
</template>