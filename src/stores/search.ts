import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import Fuse from 'fuse.js'

export interface SearchResult {
  id: string
  name: string
  path: string
  type: 'app' | 'file' | 'folder'
  icon?: string
  isRecent?: boolean
}

interface AppInfo {
  name: string
  path: string
  icon?: string
  aliases?: string[]
}

interface AppUsageData {
  count: number
  last_used: number
}

interface FileResult {
  name: string
  path: string
  size: number
  modified: string
  type: string // "file" 或 "folder"
}

export const useSearchStore = defineStore('search', () => {
  const query = ref('')
  const results = ref<SearchResult[]>([])
  const selectedIndex = ref(0)
  const apps = ref<AppInfo[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const appUsage = ref<Record<string, AppUsageData>>({})

  // Fuse.js 实例
  let fuse: Fuse<AppInfo> | null = null

  // 防抖定时器
  let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null
  const DEBOUNCE_DELAY = 150 // 150ms 防抖延迟

  // 加载自定义应用
  const loadCustomApps = async (): Promise<AppInfo[]> => {
    try {
      const customApps = await invoke<{name: string, path: string}[]>('get_custom_apps')
      return customApps.map(app => ({
        name: app.name,
        path: app.path,
        icon: undefined,
        aliases: [],
      }))
    } catch (e) {
      console.error('Failed to load custom apps:', e)
      return []
    }
  }

  // 加载使用统计
  const loadAppUsage = async () => {
    try {
      appUsage.value = await invoke<Record<string, AppUsageData>>('get_app_usage')
    } catch (e) {
      appUsage.value = {}
    }
  }

  // 检查是否是最近使用（24小时内）
  const isRecentlyUsed = (path: string): boolean => {
    const usage = appUsage.value[path]
    if (!usage || !usage.last_used) return false
    const now = Math.floor(Date.now() / 1000)
    return (now - usage.last_used) < 24 * 60 * 60 // 24小时内
  }

  // 按使用频率排序应用
  const sortAppsByUsage = (appList: AppInfo[]): AppInfo[] => {
    return [...appList].sort((a, b) => {
      const usageA = appUsage.value[a.path]?.count || 0
      const usageB = appUsage.value[b.path]?.count || 0
      return usageB - usageA // 降序
    })
  }

  // 初始化：加载应用列表
  const init = async (force = false) => {
    if (!force && apps.value.length > 0) return

    isLoading.value = true
    error.value = null
    try {
      // 并行加载系统应用、自定义应用和使用统计
      const [systemApps, customApps, _] = await Promise.all([
        invoke<AppInfo[]>('get_installed_apps'),
        loadCustomApps(),
        loadAppUsage(),
      ])

      // 合并应用列表并按使用频率排序
      apps.value = sortAppsByUsage([...customApps, ...systemApps])

      // 初始化 Fuse.js
      fuse = new Fuse(apps.value, {
        keys: ['name', 'path', 'aliases'],
        threshold: 0.4,
        includeScore: true,
        minMatchCharLength: 1,
      })
    } catch (e) {
      console.error('Failed to load apps:', e)
      error.value = '加载应用列表失败'
    } finally {
      isLoading.value = false
    }
  }

  // 执行实际搜索（内部函数）
  const executeSearch = async (keyword: string) => {
    // 如果Fuse未初始化，先初始化
    if (!fuse && apps.value.length === 0) {
      await init()
    }

    if (!fuse) {
      results.value = []
      return
    }

    // 无搜索词时：重新加载使用统计并按频率排序
    if (!keyword.trim()) {
      await loadAppUsage()
      if (appUsage.value && Object.keys(appUsage.value).length > 0) {
        apps.value = sortAppsByUsage([...apps.value])
      }
      const defaultApps = apps.value.slice(0, 12)
      results.value = defaultApps.map((app) => ({
        id: `app-${app.path}`,
        name: app.name,
        path: app.path,
        type: 'app' as const,
        isRecent: isRecentlyUsed(app.path),
      }))
      return
    }

    // 模糊搜索应用（同步，立即显示）
    const fuseResults = fuse.search(keyword, { limit: 6 })

    // 转换应用结果 - 使用路径作为 id 的一部分，避免组件复用
    const appResults: SearchResult[] = fuseResults.slice(0, 6).map((r) => ({
      id: `app-${r.item.path}`,
      name: r.item.name,
      path: r.item.path,
      type: 'app' as const,
      isRecent: isRecentlyUsed(r.item.path),
    }))

    // 先显示应用结果（无等待，立即响应）
    results.value = appResults

    // 异步搜索文件（调用 Everything SDK），搜索完成后再追加到结果中
    try {
      const files = await invoke<FileResult[]>('search_files', {
        query: keyword,
        limit: 6
      })
      const fileResults = files.map((f) => ({
        id: `file-${f.path}`,
        name: f.name,
        path: f.path,
        type: (f.type === 'folder' ? 'folder' : 'file') as 'file' | 'folder',
      }))
      // 追加文件结果
      results.value = [...appResults, ...fileResults]
    } catch (e) {
      console.error('[文件搜索错误]', e)
      error.value = typeof e === 'string' ? e : '文件搜索失败'
    }
  }

  // 搜索（带防抖）
  const search = (keyword: string) => {
    query.value = keyword
    selectedIndex.value = 0
    error.value = null

    // 清除之前的防抖定时器
    if (searchDebounceTimer) {
      clearTimeout(searchDebounceTimer)
      searchDebounceTimer = null
    }

    // 空搜索词立即执行，不防抖
    if (!keyword.trim()) {
      executeSearch(keyword)
      return
    }

    // 设置防抖定时器
    searchDebounceTimer = setTimeout(() => {
      executeSearch(keyword)
    }, DEBOUNCE_DELAY)
  }

  // 清除搜索
  const clear = async () => {
    query.value = ''
    selectedIndex.value = 0
    error.value = null
    // 清除后重新加载使用统计并显示默认应用
    if (fuse && apps.value.length > 0) {
      await loadAppUsage()
      if (appUsage.value && Object.keys(appUsage.value).length > 0) {
        apps.value = sortAppsByUsage([...apps.value])
      }
      const defaultApps = apps.value.slice(0, 12)
      results.value = defaultApps.map((app) => ({
        id: `app-${app.path}`,
        name: app.name,
        path: app.path,
        type: 'app' as const,
        isRecent: isRecentlyUsed(app.path),
      }))
    }
  }

  // 选择下一个
  const selectNext = () => {
    if (results.value.length === 0) return
    selectedIndex.value = (selectedIndex.value + 1) % results.value.length
  }

  // 选择上一个
  const selectPrev = () => {
    if (results.value.length === 0) return
    selectedIndex.value = (selectedIndex.value - 1 + results.value.length) % results.value.length
  }

  // 初始化 - 立即执行
  init()

  return {
    query,
    results,
    selectedIndex,
    isLoading,
    error,
    apps,
    search,
    clear,
    selectNext,
    selectPrev,
    init,
  }
})
