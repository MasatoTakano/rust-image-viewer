<template>
  <div class="status-bar">
    <span>{{ pageText }}</span>
    <span>{{ modeText }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { DisplayMode } from "../types";
import { usePageController } from "../composables/usePageController";
import { useImageStore } from "../composables/useImageStore";

const { currentIndex, displayMode } = usePageController();
const { entries } = useImageStore();

const pageText = computed(() => {
  const idx = currentIndex.value;
  const total = entries.value.length;
  if (total === 0) return "0 / 0";

  if (displayMode.value === DisplayMode.Spread) {
    const left = idx + 2;
    return left <= total ? `${idx + 1} - ${left} / ${total}` : `${idx + 1} / ${total}`;
  }
  return `${idx + 1} / ${total}`;
});

const modeText = computed(() =>
  displayMode.value === DisplayMode.Spread ? "見開き" : "単ページ"
);
</script>

<style scoped>
.status-bar {
  height: 28px;
  background-color: rgba(30, 30, 30, 0.9);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  font-size: 0.85em;
  color: #aaa;
  flex-shrink: 0;
}
</style>
