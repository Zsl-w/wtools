<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import SearchBar from '@/components/SearchBar/index.vue'
import ResultList from '@/components/ResultList/index.vue'
import ClipboardList from '@/components/ClipboardList/index.vue'
import FilePreview from '@/components/FilePreview/index.vue'
import { useSearchStore } from '@/stores/search'
import { useSettingsStore } from '@/stores/settings'

const searchStore = useSearchStore()
const settingsStore = useSettingsStore()

const mode = ref<'search' | 'clipboard'>('search')
const searchPlaceholder = computed(() => mode.value === 'search' ? '搜索应用或文件...' : '搜索剪贴板...')
const clipboardListRef = ref<InstanceType<typeof ClipboardList> | null>(null)
const searchBarRef = ref<InstanceType<typeof SearchBar> | null>(null)

// 错误 Toast 状态
const errorMessage = ref<string | null>(null)
const showErrorToast = ref(false)

const showErrorMessage = (message: string) => {
  errorMessage.value = message
  showErrorToast.value = true
  setTimeout(() => {
    showErrorToast.value = false
  }, 3000)
}

const selectedResult = computed(() => {
  return searchStore.results[searchStore.selectedIndex] || null
})

const showPreview = computed(() => {
  const r = selectedResult.value
  return mode.value === 'search' && r && r.type !== 'app'
})

const handleKeyDown = (e: KeyboardEvent) => {
  // Tab 键切换模式
  if (e.key === 'Tab') {
    e.preventDefault()
    e.stopPropagation()
    if (mode.value === 'search') {
      mode.value = 'clipboard'
    } else {
      mode.value = 'search'
    }
    nextTick(() => {
      searchBarRef.value?.focusInput()
    })
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
    const msg = typeof e === 'string' ? e : '打开失败'
    showErrorMessage(msg)
  }
}

let unlistenShow: (() => void) | null = null

onMounted(async () => {
  const theme = settingsStore.theme
  if (theme === 'dark' || (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
    document.documentElement.setAttribute('data-theme', 'dark')
  }

  window.addEventListener('keydown', handleKeyDown, true)

  await searchStore.search('')

  // 监听 Rust 端窗口显示完成事件，重置并聚焦
  unlistenShow = await getCurrentWindow().listen('window-shown', () => {
    mode.value = 'search'
    searchStore.selectedIndex = 0
    searchBarRef.value?.reset()
    searchBarRef.value?.focusInput()
    // 清除错误状态
    showErrorToast.value = false
    searchStore.error = null
  })
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown, true)
  if (unlistenShow) unlistenShow()
})
</script>

<template>
  <div class="main-window">
    <div class="window-content">
      <div class="search-wrapper">
        <div class="drag-region" data-tauri-drag-region></div>
        <SearchBar ref="searchBarRef" :placeholder="searchPlaceholder" />
        <div class="mode-indicator">
          <span :class="{ active: mode === 'search' }">搜索</span>
          <span class="separator">|</span>
          <span :class="{ active: mode === 'clipboard' }">剪贴板</span>
        </div>
      </div>
      <div class="result-area">
        <div class="mode-panel" :class="{ active: mode === 'search' }">
          <div class="search-content">
            <div class="search-list" :class="{ 'with-preview': showPreview }">
              <ResultList />
            </div>
            <Transition name="preview-slide">
              <div v-if="showPreview && selectedResult" class="search-preview">
                <div class="preview-divider"></div>
                <FilePreview
                  :path="selectedResult.path"
                  :name="selectedResult.name"
                  :type="selectedResult.type"
                />
              </div>
            </Transition>
          </div>
        </div>
        <div class="mode-panel" :class="{ active: mode === 'clipboard' }">
          <ClipboardList ref="clipboardListRef" />
        </div>
      </div>
    </div>

    <!-- 错误 Toast -->
    <Transition name="error-toast">
      <div v-if="showErrorToast && errorMessage" class="error-toast">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="12" />
          <line x1="12" y1="16" x2="12.01" y2="16" />
        </svg>
        <span>{{ errorMessage }}</span>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.main-window {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, rgba(20, 184, 166, 0.15) 0%, rgba(6, 182, 212, 0.1) 50%, rgba(16, 185, 129, 0.05) 100%),
              var(--glass-bg);
  border: none;
  border-radius: 8px;
  box-shadow: var(--glass-shadow),
              0 0 60px var(--accent-glow);
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
  color: var(--accent);
  background: var(--accent-subtle);
}

.mode-indicator .separator {
  opacity: 0.3;
}

.result-area {
  flex: 1;
  overflow: hidden;
  position: relative;
  background: rgba(255, 255, 255, 0.4);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.5);
}

[data-theme="dark"] .main-window {
  background: linear-gradient(135deg, rgba(20, 184, 166, 0.2) 0%, rgba(6, 182, 212, 0.15) 50%, rgba(16, 185, 129, 0.1) 100%),
              var(--glass-bg);
  box-shadow: var(--glass-shadow),
              0 0 60px var(--accent-glow);
}

[data-theme="dark"] .result-area {
  background: rgba(30, 30, 35, 0.5);
  border-color: rgba(255, 255, 255, 0.1);
}

[data-theme="dark"] .mode-indicator span.active {
  color: var(--accent-light);
  background: var(--accent-subtle);
}

/* 搜索内容区域（列表 + 预览） */
.search-content {
  display: flex;
  height: 100%;
  min-height: 0;
}

.search-list {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  transition: flex 200ms var(--ease-out);
}

.search-list.with-preview {
  flex: 0 0 55%;
}

.search-preview {
  flex: 0 0 45%;
  min-width: 0;
  display: flex;
  overflow: hidden;
}

.preview-divider {
  width: 1px;
  background: rgba(0, 0, 0, 0.06);
  flex-shrink: 0;
}

[data-theme="dark"] .preview-divider {
  background: rgba(255, 255, 255, 0.06);
}

/* 预览面板滑入动画 */
.preview-slide-enter-active {
  transition: opacity 200ms var(--ease-out), flex-basis 200ms var(--ease-out);
}

.preview-slide-leave-active {
  transition: opacity 150ms ease-in, flex-basis 150ms ease-in;
}

.preview-slide-enter-from {
  opacity: 0;
  flex-basis: 0%;
}

.preview-slide-leave-to {
  opacity: 0;
  flex-basis: 0%;
}

/* 模式面板切换 */
.mode-panel {
  position: absolute;
  inset: 0;
  opacity: 0;
  transform: translateY(6px);
  pointer-events: none;
  transition: opacity 200ms var(--ease-out), transform 200ms var(--ease-out);
  display: flex;
  flex-direction: column;
}

.mode-panel.active {
  opacity: 1;
  transform: translateY(0);
  pointer-events: auto;
}

/* 错误 Toast */
.error-toast {
  position: absolute;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(239, 68, 68, 0.95);
  color: white;
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.2);
  z-index: 1000;
  display: flex;
  align-items: center;
  gap: 8px;
  max-width: 80%;
}

.error-toast span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.error-toast-enter-active {
  transition: all 0.3s ease;
}

.error-toast-leave-active {
  transition: all 0.2s ease-in;
}

.error-toast-enter-from {
  opacity: 0;
  transform: translateX(-50%) translateY(10px);
}

.error-toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(10px);
}
</style>
