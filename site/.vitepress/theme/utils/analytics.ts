/**
 * Lightweight analytics utility
 * Respects Do Not Track and GDPR
 */

export function initAnalytics() {
  // Respect Do Not Track
  if (navigator.doNotTrack === '1') {
    console.log('[Analytics] Do Not Track enabled, skipping')
    return
  }
  
  // Check for GDPR consent
  if (!hasConsent()) {
    showConsentBanner()
    return
  }
  
  // Initialize minimal analytics
  trackPageView()
  
  // Track navigation
  if (typeof window !== 'undefined') {
    window.addEventListener('popstate', trackPageView)
  }
}

function hasConsent(): boolean {
  return localStorage.getItem('analytics-consent') === 'true'
}

function showConsentBanner() {
  // Simple consent banner could be shown here
  // For now, just log
  console.log('[Analytics] Consent not given')
}

function trackPageView() {
  // Minimal page view tracking
  const data = {
    path: window.location.pathname,
    title: document.title,
    timestamp: new Date().toISOString(),
    referrer: document.referrer,
  }
  
  // In production, send to your analytics endpoint
  if (import.meta.env.PROD) {
    // Example: send to a privacy-friendly analytics service
    // fetch('/api/analytics', { method: 'POST', body: JSON.stringify(data) })
    console.log('[Analytics] Page view:', data)
  }
}

export function trackEvent(eventName: string, properties?: Record<string, any>) {
  if (!hasConsent()) return
  
  const data = {
    event: eventName,
    properties,
    timestamp: new Date().toISOString(),
  }
  
  if (import.meta.env.PROD) {
    console.log('[Analytics] Event:', data)
  }
}
