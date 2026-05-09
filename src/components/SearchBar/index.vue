<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useSearchStore } from '@/stores/search'

defineProps<{
  placeholder?: string
}>()

const searchStore = useSearchStore()
const inputRef = ref<HTMLInputElement | null>(null)

let inputHandler: (() => void) | null = null

const focusInput = () => {
  inputRef.value?.focus()
}

const reset = () => {
  if (inputRef.value) {
    inputRef.value.value = ''
  }
  searchStore.clear()
  inputRef.value?.focus()
}

defineExpose({ focusInput, reset })

onMounted(() => {
  const el = inputRef.value
  if (!el) return

  // 原生 input 事件 — 搜索（方向键和回车由 MainWindow 在捕获阶段统一处理）
  inputHandler = () => {
    searchStore.search(el.value)
  }
  el.addEventListener('input', inputHandler)

  // 初始聚焦
  el.focus()
})

onUnmounted(() => {
  const el = inputRef.value
  if (!el) return
  if (inputHandler) el.removeEventListener('input', inputHandler)
})
</script>

<template>
  <div class="search-bar">
    <div class="search-icon-wrapper">
      <svg class="search-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <circle cx="11" cy="11" r="8" />
        <path d="m21 21-4.35-4.35" />
      </svg>
    </div>

    <input
      ref="inputRef"
      type="text"
      :placeholder="placeholder || '搜索应用或文件...'"
      autocomplete="off"
      spellcheck="false"
    />

  </div>
</template>

<style scoped>
.search-bar {
  width: 100%;
  height: 52px;
  padding: 0 6px;
  display: flex;
  align-items: center;
  gap: 10px;
  background: rgba(255, 255, 255, 0.7);
  border: 1px solid rgba(255, 255, 255, 0.6);
  border-radius: 16px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05), inset 0 1px 0 rgba(255, 255, 255, 0.8);
  transition: all 0.2s ease;
  backdrop-filter: blur(20px);
}

.search-bar:focus-within {
  background: rgba(255, 255, 255, 0.95);
  border-color: var(--selection-border);
  box-shadow: 0 4px 24px var(--accent-glow), inset 0 1px 0 rgba(255, 255, 255, 0.9);
}

.search-icon-wrapper {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-subtle);
  border-radius: 12px;
  flex-shrink: 0;
}

.search-icon {
  color: var(--accent);
  flex-shrink: 0;
}

input {
  flex: 1;
  border: none;
  background: transparent;
  font-family: var(--font-display);
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary);
  outline: none;
  letter-spacing: -0.2px;
}

input::placeholder {
  color: var(--text-tertiary);
}

.shortcut-badge {
  font-size: 11px;
  font-weight: 500;
  color: var(--text-tertiary);
  background: rgba(0, 0, 0, 0.05);
  padding: 4px 8px;
  border-radius: 8px;
  white-space: nowrap;
}

/* 暗色模式 */
[data-theme="dark"] .search-bar {
  background: rgba(30, 30, 35, 0.6);
  border-color: rgba(255, 255, 255, 0.1);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.2);
}

[data-theme="dark"] .search-bar:focus-within {
  background: rgba(40, 40, 50, 0.8);
  border-color: var(--selection-border);
  box-shadow: 0 4px 24px var(--accent-glow);
}

[data-theme="dark"] .search-icon-wrapper {
  background: var(--accent-subtle);
}

[data-theme="dark"] .search-icon {
  color: var(--accent-light);
}

[data-theme="dark"] .shortcut-badge {
  background: rgba(255, 255, 255, 0.08);
}
</style>
