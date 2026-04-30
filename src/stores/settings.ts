import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export type Theme = 'light' | 'dark' | 'system'

export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<Theme>('system')

  // 监听主题变化应用到document
  watch(theme, (newTheme) => {
    if (newTheme === 'dark') {
      document.documentElement.setAttribute('data-theme', 'dark')
    } else if (newTheme === 'light') {
      document.documentElement.removeAttribute('data-theme')
    } else {
      // system
      if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
        document.documentElement.setAttribute('data-theme', 'dark')
      } else {
        document.documentElement.removeAttribute('data-theme')
      }
    }
  }, { immediate: true })

  return {
    theme,
  }
})
