<template>
  <Transition name="notification">
    <div v-if="message" class="notification">{{ message }}</div>
  </Transition>
</template>

<script setup lang="ts">
defineProps<{ message: string }>();
</script>

<style scoped>
.notification {
  position: fixed;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  background-color: rgba(74, 158, 255, 0.9);
  color: #fff;
  padding: 10px 20px;
  border-radius: 6px;
  font-size: 0.9em;
  z-index: 2000;
}

.notification-enter-active {
  animation: fadeIn 0.2s ease-out;
}

.notification-leave-active {
  animation: fadeOut 0.3s ease-in;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateX(-50%) translateY(-10px); }
  to { opacity: 1; transform: translateX(-50%) translateY(0); }
}

@keyframes fadeOut {
  from { opacity: 1; transform: translateX(-50%) translateY(0); }
  to { opacity: 0; transform: translateX(-50%) translateY(-10px); }
}
</style>
