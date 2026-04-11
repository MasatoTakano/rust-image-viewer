<template>
  <div class="slider-bar">
    <button class="nav-btn" @click="goFirst" title="最初へ">⏮</button>
    <button class="nav-btn" @click="goLast" title="最後へ">⏭</button>
    <input
      type="range"
      :min="1"
      :max="total"
      :value="current + 1"
      @input="onSlide"
    />
  </div>
</template>

<script setup lang="ts">
import { usePageController } from "../composables/usePageController";
import { useImageStore } from "../composables/useImageStore";

const { currentIndex, setPage, goFirst, goLast } = usePageController();
const { entries } = useImageStore();

function onSlide(e: Event) {
  const val = Number((e.target as HTMLInputElement).value) - 1;
  setPage(val);
}
</script>

<style scoped>
.slider-bar {
  height: 28px;
  background-color: rgba(30, 30, 30, 0.9);
  display: flex;
  align-items: center;
  padding: 0 8px;
  flex-shrink: 0;
  gap: 6px;
}

.nav-btn {
  background: none;
  border: none;
  color: #aaa;
  font-size: 14px;
  cursor: pointer;
  padding: 2px 4px;
  line-height: 1;
  flex-shrink: 0;
}

.nav-btn:hover {
  color: #fff;
}

.slider-bar input[type="range"] {
  flex: 1;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: #444;
  border-radius: 2px;
  outline: none;
  cursor: pointer;
}

.slider-bar input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #4a9eff;
  cursor: pointer;
}

.slider-bar input[type="range"]::-webkit-slider-thumb:hover {
  background: #6ab4ff;
}
</style>
