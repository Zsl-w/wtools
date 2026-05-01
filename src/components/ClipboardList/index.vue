<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface ClipboardItem {
  id: string
  content_type: string
  preview: string
  content: string | null
  timestamp: number
  size: number
}

const items = ref<ClipboardItem[]>([])
const selectedIndex = ref(0)
const itemRefs = ref<Map<number, HTMLElement>>(new Map())
const showToast = ref(false)

const loadHistory = async () => {
  try {
    items.value = await invoke<ClipboardItem[]>('get_clipboard_history')
    selectedIndex.value = 0
  } catch (e) {
    console.error('Failed to load clipboard history:', e)
  }
}

const copyItem = async (item: ClipboardItem) => {
  try {
    if (item.content_type === 'image' && item.content) {
      // item.content 是 base64 编码的 PNG (data:image/png;base64,...)
      const base64Data = item.content.replace(/^data:image\/\w+;base64,/, '')
      await invoke('copy_image_to_clipboard', { base64Data })
    } else if (item.content_type === 'text' && item.content) {
      await invoke('copy_to_clipboard', { content: item.content })
    } else {
      return
    }
    showToast.value = true
    setTimeout(() => {
      showToast.value = false
    }, 1500)
    setTimeout(async () => {
      await invoke('hide_window')
    }, 500)
  } catch (e) {
    console.error('Failed to copy:', e)
  }
}

const deleteItem = async (id: string) => {
  try {
    await invoke('delete_clipboard_item', { id })
    await loadHistory()
  } catch (e) {
    console.error('Failed to delete:', e)
  }
}

const clearHistory = async () => {
  try {
    await invoke('clear_clipboard_history')
    items.value = []
  } catch (e) {
    console.error('Failed to clear:', e)
  }
}

const formatTime = (timestamp: number): string => {
  const date = new Date(timestamp * 1000)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)
  
  if (diffMins < 1) return '刚刚'
  if (diffMins < 60) return `${diffMins} 分钟前`
  if (diffHours < 24) return `${diffHours} 小时前`
  if (diffDays < 7) return `${diffDays} 天前`
  return date.toLocaleDateString()
}

const selectNext = () => {
  if (items.value.length > 0) {
    selectedIndex.value = (selectedIndex.value + 1) % items.value.length
  }
}

const selectPrev = () => {
  if (items.value.length > 0) {
    selectedIndex.value = (selectedIndex.value - 1 + items.value.length) % items.value.length
  }
}

const setItemRef = (index: number, el: HTMLElement | null) => {
  if (el) {
    itemRefs.value.set(index, el)
  } else {
    itemRefs.value.delete(index)
  }
}

watch(selectedIndex, async () => {
  await nextTick()
  const el = itemRefs.value.get(selectedIndex.value)
  el?.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
})

const copySelected = () => {
  const item = items.value[selectedIndex.value]
  if (item) {
    copyItem(item)
  }
}

defineExpose({ loadHistory, selectNext, selectPrev, copySelected })

onMounted(loadHistory)
</script>

<template>
  <div class="clipboard-list">
    <Transition name="toast">
      <div v-if="showToast" class="toast">复制成功</div>
    </Transition>
    <!-- 空状态 -->
    <div v-if="items.length === 0" class="empty-state">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
        <rect x="8" y="2" width="8" height="4" rx="1" ry="1" />
      </svg>
      <span>剪贴板为空</span>
      <span class="hint">复制内容后会自动记录</span>
    </div>
    
    <!-- 历史列表 -->
    <div v-else class="history-list">
      <div class="list-header">
        <span class="title">剪贴板历史</span>
        <button class="clear-btn" @click="clearHistory">清空</button>
      </div>
      
      <div class="items-container">
        <div
          v-for="(item, index) in items"
          :key="item.id"
          :ref="(el) => setItemRef(index, el as HTMLElement)"
          class="clipboard-item"
          :class="{ selected: index === selectedIndex }"
          @click="copyItem(item)"
        >
          <div class="item-content">
            <div class="item-preview" v-if="item.content_type === 'image' && item.content">
              <img :src="item.content" alt="" class="preview-image" />
            </div>
            <div class="item-preview text" v-else>{{ item.preview }}</div>
            <div class="item-time">{{ formatTime(item.timestamp) }}</div>
          </div>
          <button class="delete-btn" @click.stop="deleteItem(item.id)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6 6 18M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.clipboard-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 12px;
  color: var(--text-tertiary);
  font-size: 14px;
}

.empty-state .hint {
  font-size: 12px;
  opacity: 0.7;
}

.history-list {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

.title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}

.clear-btn {
  font-size: 12px;
  color: var(--text-tertiary);
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.clear-btn:hover {
  background: rgba(0, 0, 0, 0.05);
  color: var(--text-secondary);
}

.items-container {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.items-container::-webkit-scrollbar {
  display: none;
}

.clipboard-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px 16px;
  cursor: pointer;
  transition: all 0.15s ease;
  position: relative;
}

.clipboard-item:hover {
  background: rgba(0, 0, 0, 0.03);
}

.clipboard-item.selected {
  background: var(--selection-bg);
}

.item-content {
  flex: 1;
  min-width: 0;
}

.item-preview {
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-word;
  line-height: 1.4;
}

.item-preview.text {
  white-space: pre-wrap;
  max-height: 60px;
  overflow: hidden;
}

.preview-image {
  max-width: 200px;
  max-height: 100px;
  border-radius: 8px;
  object-fit: contain;
}

.item-time {
  font-size: 11px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

.delete-btn {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.05);
  border: none;
  border-radius: 6px;
  cursor: pointer;
  opacity: 0;
  transition: all 0.15s ease;
  color: var(--text-tertiary);
}

.clipboard-item:hover .delete-btn {
  opacity: 1;
}

.delete-btn:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

/* 暗色模式 */
[data-theme="dark"] .list-header {
  border-bottom-color: rgba(255, 255, 255, 0.05);
}

[data-theme="dark"] .clear-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

[data-theme="dark"] .clipboard-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

[data-theme="dark"] .delete-btn {
  background: rgba(255, 255, 255, 0.1);
}

.toast {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: rgba(34, 197, 94, 0.95);
  color: white;
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  display: flex;
  align-items: center;
  gap: 8px;
}

.toast::before {
  content: '✓';
  font-size: 16px;
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translate(-50%, -50%) scale(0.9);
}
</style>
