<script setup lang="ts">
</script>

<template>
  <div class="status-bar">
    <div class="shortcut-hint">
      <span class="shortcut-key">⚙️</span>
      <span>设置</span>
    </div>
    <div class="divider"></div>
    <div class="shortcut-hint">
      <span class="shortcut-key">↑↓</span>
      <span>选择</span>
    </div>
    <div class="shortcut-hint">
      <span class="shortcut-key">Enter</span>
      <span>打开</span>
    </div>
    <div class="shortcut-hint">
      <span class="shortcut-key">Alt+数字</span>
      <span>快速打开</span>
    </div>
    <div class="divider"></div>
    <div class="shortcut-hint">
      <span class="shortcut-key">Esc</span>
      <span>关闭</span>
    </div>
  </div>
</template>

<style scoped>
.status-bar {
  height: 36px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  background: rgba(0, 0, 0, 0.03);
  border-top: 1px solid rgba(0, 0, 0, 0.06);
  font-size: 11px;
  color: var(--text-tertiary);
}

[data-theme="dark"] .status-bar {
  background: rgba(255, 255, 255, 0.04);
  border-color: rgba(255, 255, 255, 0.06);
}

.shortcut-hint {
  display: flex;
  align-items: center;
  gap: 4px;
}

.shortcut-key {
  padding: 2px 6px;
  background: rgba(0, 0, 0, 0.06);
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 500;
}

[data-theme="dark"] .shortcut-key {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.12);
}

.divider {
  width: 1px;
  height: 12px;
  background: var(--glass-border);
}
</style>
