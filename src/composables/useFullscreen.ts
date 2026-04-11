import { ref, readonly } from "vue";

const isFullscreen = ref(false);

export function useFullscreen() {
  function setFullscreen(value: boolean) {
    isFullscreen.value = value;
  }

  return {
    isFullscreen: readonly(isFullscreen),
    setFullscreen,
  };
}
