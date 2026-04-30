<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useSearchStore } from '@/stores/search'

const props = defineProps<{
  modelValue: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const searchStore = useSearchStore()
const inputRef = ref<HTMLInputElement | null>(null)
const localValue = ref(props.modelValue)

const focusInput = () => {
  inputRef.value?.focus()
}

const reset = () => {
  localValue.value = ''
  emit('update:modelValue', '')
  // 延迟聚焦，确保在 Tauri 窗口焦点处理完成后再聚焦输入框
  setTimeout(() => {
    inputRef.value?.focus()
  }, 100)
}

defineExpose({ focusInput, reset })

watch(() => props.modelValue, (newVal) => {
  if (newVal !== localValue.value) {
    localValue.value = newVal
  }
})

let searchTimer: ReturnType<typeof setTimeout> | null = null

const handleInput = (e: Event) => {
  const value = (e.target as HTMLInputElement).value
  localValue.value = value
  emit('update:modelValue', value)

  if (searchTimer) {
    clearTimeout(searchTimer)
  }
  searchTimer = setTimeout(() => {
    searchStore.search(value)
  }, 150)
}

const handleClear = () => {
  localValue.value = ''
  emit('update:modelValue', '')
  searchStore.clear()
  nextTick(() => {
    inputRef.value?.focus()
  })
}

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
    if (searchStore.results.length > 0) {
      e.preventDefault()
      if (e.key === 'ArrowDown') {
        searchStore.selectNext()
      } else {
        searchStore.selectPrev()
      }
    }
    return
  }

  if (e.key === 'Enter') {
    e.preventDefault()
    if (searchStore.results.length > 0) {
      const selected = searchStore.results[searchStore.selectedIndex]
      if (selected) {
        if (selected.type === 'app') {
          invoke('launch_app', { path: selected.path })
        } else {
          invoke('open_file', { path: selected.path })
        }
        invoke('hide_window')
      }
    }
    return
  }
}

onMounted(() => {
  setTimeout(() => {
    inputRef.value?.focus()
  }, 100)
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
      v-model="localValue"
      type="text"
      @input="handleInput"
      @keydown="handleKeydown"
      placeholder="搜索应用或文件..."
      autocomplete="off"
      spellcheck="false"
    />

    <button v-if="localValue" class="clear-btn" @click="handleClear">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <path d="M18 6 6 18M6 6l12 12" />
      </svg>
    </button>
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
  border-color: rgba(99, 102, 241, 0.4);
  box-shadow: 0 4px 24px rgba(99, 102, 241, 0.12), inset 0 1px 0 rgba(255, 255, 255, 0.9);
}

.search-icon-wrapper {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(99, 102, 241, 0.1);
  border-radius: 12px;
  flex-shrink: 0;
}

.search-icon {
  color: #6366F1;
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

.clear-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: rgba(0, 0, 0, 0.08);
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.clear-btn:hover {
  background: rgba(0, 0, 0, 0.15);
  color: var(--text-primary);
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
  border-color: rgba(99, 102, 241, 0.5);
  box-shadow: 0 4px 24px rgba(99, 102, 241, 0.2);
}

[data-theme="dark"] .search-icon-wrapper {
  background: rgba(99, 102, 241, 0.2);
}

[data-theme="dark"] .search-icon {
  color: #A5B4FC;
}

[data-theme="dark"] .clear-btn {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-secondary);
}

[data-theme="dark"] .clear-btn:hover {
  background: rgba(255, 255, 255, 0.18);
}

[data-theme="dark"] .shortcut-badge {
  background: rgba(255, 255, 255, 0.08);
}
</style>
