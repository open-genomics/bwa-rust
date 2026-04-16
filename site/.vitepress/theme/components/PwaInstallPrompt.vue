<script setup lang="ts">
import { ref, onMounted } from 'vue'

const showPrompt = ref(false)
const deferredPrompt = ref<any>(null)
const isInstalled = ref(false)

onMounted(() => {
  // Check if already installed
  if (window.matchMedia('(display-mode: standalone)').matches) {
    isInstalled.value = true
    return
  }
  
  // Listen for beforeinstallprompt event
  window.addEventListener('beforeinstallprompt', (e) => {
    e.preventDefault()
    deferredPrompt.value = e
    
    // Show prompt after 5 seconds
    setTimeout(() => {
      if (!localStorage.getItem('pwa-prompt-dismissed')) {
        showPrompt.value = true
      }
    }, 5000)
  })
  
  // Listen for appinstalled event
  window.addEventListener('appinstalled', () => {
    isInstalled.value = true
    showPrompt.value = false
    deferredPrompt.value = null
  })
})

const installPwa = async () => {
  if (!deferredPrompt.value) return
  
  deferredPrompt.value.prompt()
  const { outcome } = await deferredPrompt.value.userChoice
  
  if (outcome === 'accepted') {
    console.log('User accepted PWA installation')
  }
  
  deferredPrompt.value = null
  showPrompt.value = false
}

const dismiss = () => {
  showPrompt.value = false
  localStorage.setItem('pwa-prompt-dismissed', 'true')
}
</script>

<template>
  <Transition name="slide-up">
    <div v-if="showPrompt && !isInstalled" class="pwa-prompt">
      <div class="pwa-content">
        <div class="pwa-icon">📱</div>
        <div class="pwa-text">
          <div class="pwa-title">Install bwa-rust Docs</div>
          <div class="pwa-desc">Add to home screen for offline access</div>
        </div>
      </div>
      <div class="pwa-actions">
        <button class="pwa-btn pwa-btn-install" @click="installPwa">
          Install
        </button>
        <button class="pwa-btn pwa-btn-dismiss" @click="dismiss">
          Later
        </button>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.pwa-prompt {
  position: fixed;
  bottom: 1rem;
  right: 1rem;
  left: 1rem;
  max-width: 400px;
  margin: 0 auto;
  padding: 1rem;
  background: var(--vp-c-bg);
  border: 1px solid var(--vp-c-border);
  border-radius: 16px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.pwa-content {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.pwa-icon {
  font-size: 2rem;
}

.pwa-text {
  flex: 1;
}

.pwa-title {
  font-weight: 600;
  color: var(--vp-c-text-1);
}

.pwa-desc {
  font-size: 0.875rem;
  color: var(--vp-c-text-2);
}

.pwa-actions {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
}

.pwa-btn {
  padding: 0.5rem 1rem;
  border-radius: 8px;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.pwa-btn-install {
  background: var(--vp-c-brand-1);
  color: white;
  border: none;
}

.pwa-btn-install:hover {
  background: var(--vp-c-brand-2);
}

.pwa-btn-dismiss {
  background: transparent;
  color: var(--vp-c-text-2);
  border: 1px solid var(--vp-c-border);
}

.pwa-btn-dismiss:hover {
  background: var(--vp-c-bg-soft);
}

/* Animation */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.3s ease;
}

.slide-up-enter-from,
.slide-up-leave-to {
  transform: translateY(100%);
  opacity: 0;
}

@media (min-width: 450px) {
  .pwa-prompt {
    left: auto;
    right: 1rem;
  }
}
</style>
