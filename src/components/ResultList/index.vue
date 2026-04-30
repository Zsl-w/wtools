<script setup lang="ts">
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useSearchStore } from '@/stores/search'
import ResultItem from '@/components/ResultItem/index.vue'

const searchStore = useSearchStore()

const appResults = computed(() => searchStore.results.filter(r => r.type === 'app'))
const fileResults = computed(() => searchStore.results.filter(r => r.type === 'file' || r.type === 'folder'))
const hasQuery = computed(() => searchStore.query.trim().length > 0)

const isAdding = ref(false)

const addCustomApp = async () => {
  if (isAdding.value) return
  
  const file = await open({
    multiple: false,
    filters: [{ name: '应用程序', extensions: ['exe'] }],
  })
  
  if (file && typeof file === 'string') {
    isAdding.value = true
    try {
      const name = file.split(/[\\/]/).pop()?.replace('.exe', '') || '自定义应用'
      await invoke('add_custom_app', { name, path: file })
      await searchStore.init(true) // 强制重新加载
      await searchStore.search('') // 刷新搜索结果
    } catch (e) {
      console.error('Failed to add custom app:', e)
    } finally {
      isAdding.value = false
    }
  }
}
</script>

<template>
  <div class="result-list">
    <!-- 加载中 -->
    <div v-if="searchStore.isLoading" class="empty-state">
      <div class="loading">加载应用中...</div>
    </div>

    <!-- 无搜索结果 -->
    <div v-else-if="hasQuery && searchStore.results.length === 0" class="empty-state">
      <div class="no-results">未找到匹配结果</div>
    </div>

    <!-- 应用结果 -->
    <div v-if="appResults.length > 0" class="result-section">
      <div class="section-title">{{ hasQuery ? '应用程序' : '常用应用' }}</div>
      <div class="section-items">
        <ResultItem
          v-for="(item, index) in appResults"
          :key="item.id"
          :item="item"
          :index="index"
        />
      </div>
    </div>

    <!-- 文件结果 -->
    <div v-if="hasQuery && fileResults.length > 0" class="result-section">
      <div class="section-title">文件与文件夹</div>
      <div class="section-items">
        <ResultItem
          v-for="(item, index) in fileResults"
          :key="item.id"
          :item="item"
          :index="appResults.length + index"
        />
      </div>
    </div>

    <!-- 添加自定义应用 -->
    <div class="add-app-section">
      <button class="add-app-btn" @click="addCustomApp" :disabled="isAdding">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
        <span>{{ isAdding ? '添加中...' : '添加应用' }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.result-list {
  display: flex;
  flex-direction: column;
  gap: 0;
  padding: 8px 0;
  height: 100%;
  overflow-y: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.result-list::-webkit-scrollbar {
  display: none;
}

.result-section {
  display: flex;
  flex-direction: column;
  padding: 0 12px;
}

.result-section + .result-section {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid rgba(0, 0, 0, 0.05);
}

.section-title {
  font-family: var(--font-display);
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  padding: 10px 12px 6px;
  text-transform: none;
  letter-spacing: 0.3px;
}

.section-items {
  display: flex;
  flex-direction: column;
  position: relative;
}

[data-theme="dark"] .result-list {
  scrollbar-color: rgba(255, 255, 255, 0.2) transparent;
}

[data-theme="dark"] .result-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
}

[data-theme="dark"] .result-section + .result-section {
  border-top-color: rgba(255, 255, 255, 0.05);
}



.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px;
  color: var(--text-tertiary);
  font-size: 14px;
}

.loading {
  display: flex;
  align-items: center;
  gap: 8px;
}

.loading::after {
  content: '';
  width: 16px;
  height: 16px;
  border: 2px solid var(--gray-300);
  border-top-color: var(--primary-500);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.no-results {
  opacity: 0.6;
}

.add-app-section {
  padding: 12px 16px;
  border-top: 1px solid rgba(0, 0, 0, 0.04);
  margin-top: 8px;
}

.add-app-btn {
  width: 100%;
  padding: 10px 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-tertiary);
  background: transparent;
  border: 1px dashed rgba(0, 0, 0, 0.15);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.add-app-btn:hover {
  color: var(--text-secondary);
  border-color: rgba(0, 0, 0, 0.25);
  background: rgba(0, 0, 0, 0.02);
}

.add-app-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-theme="dark"] .add-app-section {
  border-top-color: rgba(255, 255, 255, 0.04);
}

[data-theme="dark"] .add-app-btn {
  border-color: rgba(255, 255, 255, 0.1);
  color: var(--text-tertiary);
}

[data-theme="dark"] .add-app-btn:hover {
  border-color: rgba(255, 255, 255, 0.2);
  background: rgba(255, 255, 255, 0.05);
}
</style>
