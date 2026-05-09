<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface Props {
  path: string
  name: string
  type: 'file' | 'folder' | 'app'
}

const props = defineProps<Props>()

interface FilePreviewResult {
  preview_type: string
  content: string | null
  total_lines: number | null
  size: number
  modified: string
  extension: string
}

const preview = ref<FilePreviewResult | null>(null)
const imageSrc = ref<string | null>(null)
const loading = ref(false)

const isImage = computed(() => {
  const ext = props.name.split('.').pop()?.toLowerCase() || ''
  return ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'ico', 'svg'].includes(ext)
})

const formatSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

const loadPreview = async () => {
  if (!props.path || props.type === 'app') {
    preview.value = null
    imageSrc.value = null
    return
  }

  loading.value = true
  preview.value = null
  imageSrc.value = null

  try {
    if (isImage.value) {
      const thumb = await invoke<string | null>('get_image_thumbnail', { path: props.path })
      if (thumb) {
        imageSrc.value = thumb
      }
      // Also get metadata
      const result = await invoke<FilePreviewResult | null>('get_file_preview', { path: props.path })
      if (result) {
        preview.value = result
      }
    } else {
      const result = await invoke<FilePreviewResult | null>('get_file_preview', { path: props.path })
      if (result) {
        preview.value = result
      }
    }
  } catch (e) {
    console.error('Failed to load preview:', e)
  } finally {
    loading.value = false
  }
}

watch(() => props.path, loadPreview, { immediate: true })

const highlightedLines = computed(() => {
  if (!preview.value?.content) return []
  return preview.value.content.split('\n')
})

const lineCountOffset = computed(() => {
  return highlightedLines.value.length
})

const getLanguageTag = (ext: string): string => {
  const map: Record<string, string> = {
    js: 'JavaScript', ts: 'TypeScript', jsx: 'React', tsx: 'TypeScript',
    vue: 'Vue', svelte: 'Svelte',
    rs: 'Rust', py: 'Python', rb: 'Ruby', go: 'Go', java: 'Java',
    c: 'C', cpp: 'C++', h: 'C Header', hpp: 'C++ Header', cs: 'C#',
    html: 'HTML', htm: 'HTML', css: 'CSS', scss: 'SCSS', less: 'LESS',
    json: 'JSON', xml: 'XML', yaml: 'YAML', yml: 'YAML', toml: 'TOML',
    md: 'Markdown', txt: 'Text', log: 'Log',
    sh: 'Shell', bash: 'Bash', ps1: 'PowerShell', bat: 'Batch',
    sql: 'SQL', graphql: 'GraphQL', proto: 'Protobuf',
    pdf: 'PDF 文档', docx: 'Word 文档', xlsx: 'Excel 表格', pptx: 'PPT 演示',
  }
  return map[ext] || ext.toUpperCase()
}
</script>

<template>
  <div class="file-preview" v-if="type !== 'app'">
    <!-- Loading -->
    <div v-if="loading" class="preview-loading">
      <div class="loading-spinner"></div>
    </div>

    <!-- Folder preview -->
    <div v-else-if="type === 'folder'" class="preview-folder">
      <div class="folder-icon">📁</div>
      <div class="folder-name">{{ name }}</div>
      <div class="folder-path">{{ path }}</div>
    </div>

    <!-- Image preview -->
    <div v-else-if="isImage && imageSrc" class="preview-image-container">
      <img :src="imageSrc" class="preview-image" :alt="name" />
      <div class="preview-meta-bar">
        <span class="meta-tag">{{ preview?.extension?.toUpperCase() || 'IMG' }}</span>
        <span class="meta-size">{{ preview ? formatSize(preview.size) : '' }}</span>
        <span class="meta-date">{{ preview?.modified }}</span>
      </div>
    </div>

    <!-- Text/code preview -->
    <div v-else-if="preview?.preview_type === 'text'" class="preview-text-container">
      <div class="preview-header">
        <span class="lang-tag">{{ getLanguageTag(preview.extension) }}</span>
        <span class="line-info">{{ preview.total_lines }} 行 · {{ formatSize(preview.size) }}</span>
      </div>
      <div class="preview-code">
        <div class="line-numbers">
          <div v-for="i in lineCountOffset" :key="i" class="line-num">{{ i }}</div>
        </div>
        <pre class="code-content"><code>{{ preview.content }}</code></pre>
      </div>
    </div>

    <!-- Binary/unknown file -->
    <div v-else class="preview-binary">
      <div class="binary-icon">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
          <polyline points="14 2 14 8 20 8" />
        </svg>
      </div>
      <div class="binary-name">{{ name }}</div>
      <div class="binary-meta" v-if="preview">
        <div class="meta-row">
          <span class="meta-label">类型</span>
          <span class="meta-value">{{ preview.extension.toUpperCase() }} 文件</span>
        </div>
        <div class="meta-row">
          <span class="meta-label">大小</span>
          <span class="meta-value">{{ formatSize(preview.size) }}</span>
        </div>
        <div class="meta-row">
          <span class="meta-label">修改时间</span>
          <span class="meta-value">{{ preview.modified }}</span>
        </div>
      </div>
      <div class="binary-hint">无法预览此文件类型</div>
    </div>
  </div>
</template>

<style scoped>
.file-preview {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* Loading */
.preview-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.loading-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--gray-300, #d1d5db);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Folder */
.preview-folder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 8px;
  padding: 24px;
}

.folder-icon {
  font-size: 40px;
  line-height: 1;
}

.folder-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  text-align: center;
  word-break: break-all;
}

.folder-path {
  font-size: 11px;
  color: var(--text-tertiary);
  text-align: center;
  word-break: break-all;
  max-width: 100%;
}

/* Image */
.preview-image-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.preview-image {
  flex: 1;
  object-fit: contain;
  width: 100%;
  min-height: 0;
  padding: 8px;
  box-sizing: border-box;
}

.preview-meta-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border-top: 1px solid rgba(0, 0, 0, 0.06);
  font-size: 11px;
  color: var(--text-tertiary);
}

[data-theme="dark"] .preview-meta-bar {
  border-top-color: rgba(255, 255, 255, 0.06);
}

.meta-tag {
  font-size: 10px;
  font-weight: 600;
  color: var(--accent);
  background: var(--accent-subtle);
  padding: 1px 5px;
  border-radius: 3px;
}

.meta-size, .meta-date {
  font-size: 11px;
}

/* Text/code */
.preview-text-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.06);
  flex-shrink: 0;
}

[data-theme="dark"] .preview-header {
  border-bottom-color: rgba(255, 255, 255, 0.06);
}

.lang-tag {
  font-size: 10px;
  font-weight: 600;
  color: var(--accent);
  background: var(--accent-subtle);
  padding: 2px 6px;
  border-radius: 3px;
}

.line-info {
  font-size: 11px;
  color: var(--text-tertiary);
}

.preview-code {
  flex: 1;
  display: flex;
  overflow: auto;
  min-height: 0;
  scrollbar-width: thin;
}

.preview-code::-webkit-scrollbar {
  width: 4px;
  height: 4px;
}

.preview-code::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.15);
  border-radius: 2px;
}

[data-theme="dark"] .preview-code::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
}

.line-numbers {
  flex-shrink: 0;
  padding: 10px 0;
  text-align: right;
  user-select: none;
  border-right: 1px solid rgba(0, 0, 0, 0.06);
}

[data-theme="dark"] .line-numbers {
  border-right-color: rgba(255, 255, 255, 0.06);
}

.line-num {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 11px;
  line-height: 1.6;
  color: var(--text-tertiary);
  opacity: 0.5;
  padding: 0 8px;
  min-width: 24px;
}

.code-content {
  flex: 1;
  margin: 0;
  padding: 10px 12px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 11.5px;
  line-height: 1.6;
  color: var(--text-primary);
  white-space: pre;
  tab-size: 4;
  min-width: 0;
}

.code-content code {
  font-family: inherit;
}

/* Binary/unknown */
.preview-binary {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 12px;
  padding: 24px;
  color: var(--text-tertiary);
}

.binary-icon {
  opacity: 0.4;
}

.binary-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  text-align: center;
  word-break: break-all;
  max-width: 100%;
}

.binary-meta {
  width: 100%;
  max-width: 200px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px;
  background: rgba(0, 0, 0, 0.03);
  border-radius: 8px;
}

[data-theme="dark"] .binary-meta {
  background: rgba(255, 255, 255, 0.04);
}

.meta-row {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
}

.meta-label {
  color: var(--text-tertiary);
}

.meta-value {
  color: var(--text-secondary);
  font-weight: 500;
}

.binary-hint {
  font-size: 11px;
  opacity: 0.6;
}

/* Dark mode */
[data-theme="dark"] .preview-loading .loading-spinner {
  border-color: var(--gray-600, #4b5563);
  border-top-color: var(--accent-light);
}
</style>
