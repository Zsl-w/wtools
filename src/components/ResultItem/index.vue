<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue'
import { useSearchStore } from '@/stores/search'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
  item: {
    id: string
    name: string
    path: string
    type: 'app' | 'file' | 'folder'
    icon?: string
    isRecent?: boolean
  }
  index: number
}>()

const searchStore = useSearchStore()
const isSelected = computed(() => searchStore.selectedIndex === props.index)
const iconUrl = ref<string | null>(null)
const itemRef = ref<HTMLElement | null>(null)

const loadIcon = async () => {
  iconUrl.value = null

  if (props.item.type === 'app') {
    try {
      const icon = await invoke<string | null>('get_app_icon_base64', { path: props.item.path })
      if (icon) {
        iconUrl.value = icon
      }
    } catch (e) {
      // 忽略错误
    }
  } else if (props.item.type === 'file') {
    // 检查是否是图片文件
    const path = props.item.path.toLowerCase()
    const imageExtensions = ['.jpg', '.jpeg', '.png', '.gif', '.webp', '.bmp', '.ico', '.svg']
    const isImage = imageExtensions.some(ext => path.endsWith(ext))

    if (isImage) {
      try {
        const thumbnail = await invoke<string | null>('get_image_thumbnail', { path: props.item.path })
        if (thumbnail) {
          iconUrl.value = thumbnail
        }
      } catch (e) {
        // 忽略错误
      }
    }
  }
}

onMounted(loadIcon)

watch(() => props.item.path, (newPath, oldPath) => {
  if (newPath !== oldPath) {
    loadIcon()
  }
})

const handleOpen = async () => {
  try {
    if (props.item.type === 'app') {
      await invoke('launch_app', { path: props.item.path })
    } else if (props.item.type === 'folder') {
      await invoke('open_file', { path: props.item.path })
    } else {
      await invoke('open_file', { path: props.item.path })
    }
    await invoke('hide_window')
  } catch (e) {
    console.error('Failed to open:', e)
  }
}

watch(isSelected, (selected) => {
  if (selected && itemRef.value) {
    itemRef.value.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
  }
})

// 获取类型标识点颜色
const getTypeColor = (item: typeof props.item): string => {
  if (item.type === 'app') return 'var(--type-app)'
  if (item.type === 'folder') return '#F59E0B' // 橙色表示文件夹

  const path = item.path.toLowerCase()
  if (path.endsWith('.doc') || path.endsWith('.docx')) return 'var(--type-doc)'
  if (path.endsWith('.pdf')) return 'var(--type-doc)'
  if (path.endsWith('.jpg') || path.endsWith('.png') || path.endsWith('.gif') || path.endsWith('.webp')) return 'var(--type-image)'
  if (path.endsWith('.mp3') || path.endsWith('.mp4') || path.endsWith('.avi') || path.endsWith('.mkv')) return 'var(--type-media)'
  if (path.endsWith('.zip') || path.endsWith('.rar') || path.endsWith('.7z')) return 'var(--type-link)'

  return 'var(--type-file)'
}

// 获取类型标签文字
const getTypeLabel = (item: typeof props.item): string => {
  if (item.type === 'app') return '应用'
  if (item.type === 'folder') return '文件夹'

  const path = item.path.toLowerCase()
  if (path.endsWith('.doc') || path.endsWith('.docx')) return '文档'
  if (path.endsWith('.pdf')) return 'PDF'
  if (path.endsWith('.xls') || path.endsWith('.xlsx')) return '表格'
  if (path.endsWith('.ppt') || path.endsWith('.pptx')) return '演示'
  if (path.endsWith('.jpg') || path.endsWith('.png') || path.endsWith('.gif')) return '图片'
  if (path.endsWith('.mp3') || path.endsWith('.wav')) return '音频'
  if (path.endsWith('.mp4') || path.endsWith('.avi') || path.endsWith('.mkv')) return '视频'
  if (path.endsWith('.zip') || path.endsWith('.rar') || path.endsWith('.7z')) return '压缩'
  if (path.endsWith('.txt') || path.endsWith('.md')) return '文本'
  if (path.endsWith('.js') || path.endsWith('.ts') || path.endsWith('.vue') || path.endsWith('.html')) return '代码'

  return '文件'
}

const getIconEmoji = (item: typeof props.item) => {
  if (item.type === 'folder') return '📁'
  const path = item.path.toLowerCase()
  if (item.type === 'app' || path.endsWith('.exe')) return '⚙️'
  if (path.endsWith('.doc') || path.endsWith('.docx')) return '📝'
  if (path.endsWith('.xls') || path.endsWith('.xlsx')) return '📊'
  if (path.endsWith('.ppt') || path.endsWith('.pptx')) return '📽️'
  if (path.endsWith('.pdf')) return '📕'
  if (path.endsWith('.zip') || path.endsWith('.rar') || path.endsWith('.7z')) return '📦'
  if (path.endsWith('.jpg') || path.endsWith('.png') || path.endsWith('.gif') || path.endsWith('.webp')) return '🖼️'
  if (path.endsWith('.mp3') || path.endsWith('.wav')) return '🎵'
  if (path.endsWith('.mp4') || path.endsWith('.avi') || path.endsWith('.mkv')) return '🎬'
  if (path.endsWith('.txt')) return '📄'
  if (path.endsWith('.md')) return '📋'
  if (path.endsWith('.js') || path.endsWith('.ts') || path.endsWith('.vue') || path.endsWith('.jsx')) return '💻'
  if (path.endsWith('.html') || path.endsWith('.htm')) return '🌐'
  if (path.endsWith('.css')) return '🎨'
  return '📄'
}

const escapeHtml = (str: string): string => {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;')
}

const highlightedName = computed(() => {
  const query = searchStore.query.toLowerCase()
  const name = props.item.name
  if (!query) return escapeHtml(name)

  const lowerName = name.toLowerCase()
  const index = lowerName.indexOf(query)
  if (index === -1) return escapeHtml(name)

  const escapedName = escapeHtml(name)
  const escapedQuery = escapeHtml(query)

  return escapedName.slice(0, index) +
    `<mark>${escapedName.slice(index, index + escapedQuery.length)}</mark>` +
    escapedName.slice(index + escapedQuery.length)
})
</script>

<template>
  <div
    ref="itemRef"
    class="result-item"
    :class="{ selected: isSelected }"
    @click="handleOpen"
  >
    <!-- 彩色类型标识点 -->
    <div class="type-indicator" :style="{ backgroundColor: getTypeColor(item) }"></div>
    
    <!-- 图标 -->
    <div class="item-icon" :class="{ 'image-thumbnail': item.type === 'file' && iconUrl }">
      <img v-if="iconUrl" :src="iconUrl" class="app-icon" :class="{ 'image-thumb': item.type === 'file' }" alt="" />
      <span v-else class="icon-emoji">{{ getIconEmoji(item) }}</span>
    </div>
    
    <!-- 信息区域 -->
    <div class="item-info">
      <div class="item-name" v-html="highlightedName"></div>
      <div class="item-meta">
        <span class="type-label">{{ getTypeLabel(item) }}</span>
        <span v-if="item.type === 'file' || item.type === 'folder'" class="item-path">{{ item.path }}</span>
      </div>
    </div>
    
    <!-- 快捷键提示 -->
    <div v-if="isSelected" class="shortcut-hint">↵</div>
  </div>
</template>

<style scoped>
.result-item {
  height: 56px;
  padding: 8px 16px;
  margin: 4px 12px;
  display: flex;
  align-items: center;
  gap: 12px;
  border-radius: 12px;
  cursor: pointer;
  position: relative;
  background: rgba(255, 255, 255, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.3);
  transition: all 180ms ease;
}

.result-item:hover {
  background: rgba(255, 255, 255, 0.8);
  transform: translateX(4px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
}

.result-item.selected {
  background: rgba(99, 102, 241, 0.12);
  border-color: rgba(99, 102, 241, 0.3);
  box-shadow: 0 4px 16px rgba(99, 102, 241, 0.15);
}

/* 类型标识点 */
.type-indicator {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
  box-shadow: 0 0 6px currentColor;
}

/* 图标容器 */
.item-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.8);
  border-radius: 10px;
  flex-shrink: 0;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.app-icon {
  width: 24px;
  height: 24px;
  object-fit: contain;
  border-radius: 4px;
}

/* 图片缩略图样式 */
.item-icon.image-thumbnail {
  padding: 0;
}

.app-icon.image-thumb {
  width: 36px;
  height: 36px;
  object-fit: cover;
  border-radius: 8px;
}

.icon-emoji {
  font-size: 18px;
  line-height: 1;
}

/* 信息区域 */
.item-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.item-name {
  font-family: var(--font-body);
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-name :deep(mark) {
  background: rgba(99, 102, 241, 0.2);
  color: #6366F1;
  border-radius: 3px;
  padding: 0 3px;
}

.item-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.type-label {
  font-size: 11px;
  font-weight: 500;
  color: var(--text-tertiary);
  background: rgba(0, 0, 0, 0.05);
  padding: 1px 6px;
  border-radius: 4px;
}

.item-path {
  font-size: 11px;
  color: var(--text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

/* 快捷键提示 */
.shortcut-hint {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  color: #6366F1;
  background: rgba(99, 102, 241, 0.15);
  border-radius: 6px;
}

/* 暗色模式 */
[data-theme="dark"] .result-item {
  background: rgba(30, 30, 35, 0.4);
  border-color: rgba(255, 255, 255, 0.08);
}

[data-theme="dark"] .result-item:hover {
  background: rgba(50, 50, 60, 0.6);
}

[data-theme="dark"] .result-item.selected {
  background: rgba(99, 102, 241, 0.2);
  border-color: rgba(99, 102, 241, 0.4);
}

[data-theme="dark"] .item-icon {
  background: rgba(255, 255, 255, 0.1);
}

[data-theme="dark"] .type-label {
  background: rgba(255, 255, 255, 0.1);
}

[data-theme="dark"] .item-name :deep(mark) {
  background: rgba(99, 102, 241, 0.3);
  color: #A5B4FC;
}

[data-theme="dark"] .shortcut-hint {
  background: rgba(99, 102, 241, 0.25);
  color: #A5B4FC;
}

/* 动画 */
.result-item-enter-active {
  transition: opacity 180ms ease-out, transform 180ms ease-out;
}

.result-item-leave-active {
  transition: opacity 100ms ease-in;
}

.result-item-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.result-item-leave-to {
  opacity: 0;
}
</style>
