import { ref, readonly, watch } from "vue";
import { DisplayMode } from "../types";
import { useImageStore } from "./useImageStore";

const currentIndex = ref(0);
const displayMode = ref<DisplayMode>(DisplayMode.Single);

export function usePageController() {
  const { entries, preloadAround } = useImageStore();

  watch(currentIndex, (idx) => {
    if (entries.value.length > 0) {
      preloadAround(idx);
    }
  });

  function setPage(index: number) {
    if (entries.value.length === 0) return;
    currentIndex.value = Math.max(0, Math.min(index, entries.value.length - 1));
  }

  function nextPage() {
    const step = displayMode.value === DisplayMode.Spread ? 2 : 1;
    const next = currentIndex.value + step;
    if (next < entries.value.length) {
      setPage(next);
    } else {
      setPage(0);
    }
  }

  function prevPage() {
    const step = displayMode.value === DisplayMode.Spread ? 2 : 1;
    const prev = currentIndex.value - step;
    if (prev >= 0) {
      setPage(prev);
    } else {
      goLast();
    }
  }

  function goFirst() {
    setPage(0);
  }

  function goLast() {
    if (entries.value.length === 0) return;
    if (displayMode.value === DisplayMode.Spread) {
      const total = entries.value.length;
      setPage(total - (total % 2 === 0 ? 2 : 1));
    } else {
      setPage(entries.value.length - 1);
    }
  }

  function toggleDisplayMode() {
    displayMode.value =
      displayMode.value === DisplayMode.Single
        ? DisplayMode.Spread
        : DisplayMode.Single;
  }

  function resetIndex() {
    currentIndex.value = 0;
  }

  function setDisplayMode(mode: DisplayMode) {
    displayMode.value = mode;
  }

  return {
    currentIndex: readonly(currentIndex),
    displayMode: readonly(displayMode),
    setPage,
    nextPage,
    prevPage,
    goFirst,
    goLast,
    toggleDisplayMode,
    resetIndex,
    setDisplayMode,
  };
}
