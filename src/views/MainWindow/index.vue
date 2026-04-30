<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import SearchBar from '@/components/SearchBar/index.vue'
import ResultList from '@/components/ResultList/index.vue'
import ClipboardList from '@/components/ClipboardList/index.vue'
import { useSearchStore } from '@/stores/search'
import { useSettingsStore } from '@/stores/settings'

const searchStore = useSearchStore()
const settingsStore = useSettingsStore()

const query = ref('')
const mode = ref<'search' | 'clipboard'>('search')
const clipboardListRef = ref<InstanceType<typeof ClipboardList> | null>(null)
const searchBarRef = ref<InstanceType<typeof SearchBar> | null>(null)

const handleKeyDown = (e: KeyboardEvent) => {
  // Tab 键切换模式
  if (e.key === 'Tab') {
    e.preventDefault()
    e.stopPropagation()
    if (mode.value === 'search') {
      mode.value = 'clipboard'
      clipboardListRef.value?.loadHistory()
    } else {
      mode.value = 'search'
    }
    return
  }

  if (e.key === 'Escape') {
    e.preventDefault()
    e.stopPropagation()
    invoke('hide_window')
    return
  }

  if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
    e.preventDefault()
    e.stopPropagation()
    if (mode.value === 'search') {
      if (searchStore.results.length > 0) {
        if (e.key === 'ArrowDown') {
          searchStore.selectNext()
        } else {
          searchStore.selectPrev()
        }
      }
    } else {
      if (e.key === 'ArrowDown') {
        clipboardListRef.value?.selectNext()
      } else {
        clipboardListRef.value?.selectPrev()
      }
    }
    return
  }

  if (e.key === 'Enter') {
    e.preventDefault()
    e.stopPropagation()
    if (mode.value === 'search' && searchStore.results.length > 0) {
      handleOpenSelected()
    } else if (mode.value === 'clipboard') {
      clipboardListRef.value?.copySelected()
    }
    return
  }
}

const handleOpenSelected = async () => {
  const selected = searchStore.results[searchStore.selectedIndex]
  if (!selected) return

  try {
    if (selected.type === 'app') {
      await invoke('launch_app', { path: selected.path })
    } else {
      await invoke('open_file', { path: selected.path })
    }
    await invoke('hide_window')
  } catch (e) {
    console.error('Failed to open:', e)
  }
}

const resetOnShow = () => {
  query.value = ''
  mode.value = 'search'
  searchStore.selectedIndex = 0  // 只重置选中索引，不清空 results
  searchBarRef.value?.reset()
}

let unlistenFn: (() => void) | null = null

onMounted(async () => {
  const theme = settingsStore.theme
  if (theme === 'dark' || (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
    document.documentElement.setAttribute('data-theme', 'dark')
  }

  window.addEventListener('keydown', handleKeyDown, true)

  await searchStore.search('')

  unlistenFn = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
    if (focused) {
      resetOnShow()
    }
  })
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown, true)
  if (unlistenFn) {
    unlistenFn()
  }
})
</script>

<template>
  <div class="main-window">
    <div class="window-content">
      <div class="search-wrapper">
        <div class="drag-region" data-tauri-drag-region></div>
        <SearchBar ref="searchBarRef" v-model="query" />
        <div class="mode-indicator">
          <span :class="{ active: mode === 'search' }">搜索</span>
          <span class="separator">|</span>
          <span :class="{ active: mode === 'clipboard' }">剪贴板</span>
        </div>
      </div>
      <div class="result-area">
        <ResultList v-if="mode === 'search'" />
        <ClipboardList v-else ref="clipboardListRef" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.main-window {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.15) 0%, rgba(168, 85, 247, 0.1) 50%, rgba(236, 72, 153, 0.05) 100%),
              var(--glass-bg);
  border: none;
  border-radius: 24px;
  box-shadow: var(--glass-shadow),
              0 0 60px rgba(99, 102, 241, 0.15);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  backdrop-filter: var(--glass-blur);
  -webkit-backdrop-filter: var(--glass-blur);
}

.window-content {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: 100%;
  box-sizing: border-box;
}

.search-wrapper {
  position: relative;
}

.drag-region {
  position: absolute;
  top: -16px;
  left: 0;
  right: 0;
  height: 32px;
  cursor: grab;
}

.drag-region:active {
  cursor: grabbing;
}

.mode-indicator {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 11px;
  color: var(--text-tertiary);
  display: flex;
  align-items: center;
  gap: 4px;
}

.mode-indicator span {
  padding: 2px 6px;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.mode-indicator span.active {
  color: #6366F1;
  background: rgba(99, 102, 241, 0.1);
}

.mode-indicator .separator {
  opacity: 0.3;
}

.result-area {
  flex: 1;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.4);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.5);
}

[data-theme="dark"] .main-window {
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.2) 0%, rgba(168, 85, 247, 0.15) 50%, rgba(236, 72, 153, 0.1) 100%),
              var(--glass-bg);
  box-shadow: var(--glass-shadow),
              0 0 60px rgba(99, 102, 241, 0.2);
}

[data-theme="dark"] .result-area {
  background: rgba(30, 30, 35, 0.5);
  border-color: rgba(255, 255, 255, 0.1);
}

[data-theme="dark"] .mode-indicator span.active {
  color: #A5B4FC;
  background: rgba(99, 102, 241, 0.2);
}
</style>
