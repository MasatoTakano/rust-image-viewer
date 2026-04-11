<template>
  <div class="viewer" @click="handleClick" @contextmenu.prevent="handleRightClick" @wheel.prevent="handleWheel">
    <div v-if="loading" class="spinner-overlay">
      <div class="spinner"></div>
    </div>
    <div class="image-container" :class="{ spread: isSpread }">
      <template v-if="isSpread">
        <img v-if="leftUrl" :src="leftUrl" draggable="false" class="spread-left" />
        <div v-else class="image-placeholder"></div>
        <img v-if="rightUrl" :src="rightUrl" draggable="false" class="spread-right" />
        <div v-else class="image-placeholder">画像を読み込めません</div>
      </template>
      <template v-else>
        <img v-if="singleUrl" :src="singleUrl" draggable="false" />
        <div v-else class="image-placeholder">画像を読み込めません</div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { DisplayMode } from "../types";
import { useImageStore } from "../composables/useImageStore";
import { usePageController } from "../composables/usePageController";
import { useSettings } from "../composables/useSettings";

const { entries, getImage, refreshInBackground, cacheHas } = useImageStore();
const { currentIndex, displayMode, nextPage, prevPage } = usePageController();
const { settings } = useSettings();

const loading = ref(false);
const isSpread = ref(false);
const singleUrl = ref<string | null>(null);
const leftUrl = ref<string | null>(null);
const rightUrl = ref<string | null>(null);

let renderGeneration = 0;

async function render() {
  const gen = ++renderGeneration;
  const idx = currentIndex.value;
  isSpread.value = displayMode.value === DisplayMode.Spread;

  const isCached = isSpread.value
    ? cacheHas(idx) && (idx + 1 >= entries.value.length || cacheHas(idx + 1))
    : cacheHas(idx);

  if (!isCached) {
    loading.value = true;
    if (isSpread.value) {
      rightUrl.value = null;
      leftUrl.value = null;
    } else {
      singleUrl.value = null;
    }
  }

  if (isSpread.value) {
    const rightData = await getImage(idx);
    if (gen !== renderGeneration) { loading.value = false; return; }
    rightUrl.value = rightData;
    const leftIdx = idx + 1;
    const leftData = leftIdx < entries.value.length ? await getImage(leftIdx) : null;
    if (gen !== renderGeneration) { loading.value = false; return; }
    leftUrl.value = leftData;
  } else {
    const data = await getImage(idx);
    if (gen !== renderGeneration) { loading.value = false; return; }
    singleUrl.value = data;
  }
  loading.value = false;
}

watch([currentIndex, displayMode, () => entries.value], () => {
  render();
}, { immediate: true });

let lastWheelTime = 0;

function handleClick() {
  nextPage();
}

function handleRightClick() {
  prevPage();
}

function handleWheel(e: WheelEvent) {
  const now = Date.now();
  if (now - lastWheelTime < settings.value.wheel_throttle_ms) return;
  lastWheelTime = now;
  if (e.deltaY > 0) nextPage();
  else if (e.deltaY < 0) prevPage();
}

function refreshAfterResize() {
  refreshInBackground(currentIndex.value);
  render();
}

defineExpose({ render, refreshAfterResize });
</script>

<style scoped>
.viewer {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  position: relative;
}

.image-container {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}

.image-container img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.image-container.spread img {
  width: 50%;
  height: 100%;
  object-fit: contain;
}

.image-container.spread img.spread-left {
  object-position: right center;
}

.image-container.spread img.spread-right {
  object-position: left center;
}

.image-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #666;
  font-size: 0.9em;
  min-width: 200px;
  min-height: 200px;
}

.spinner-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
  pointer-events: none;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid rgba(255, 255, 255, 0.2);
  border-top-color: rgba(255, 255, 255, 0.8);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
