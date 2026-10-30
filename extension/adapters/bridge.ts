export {};
// Browser interoperability only. Validation and all application policy live in Rust.
const api = globalThis.browser ?? chrome;
globalThis.limitlessBridge = {
  record_changed: () => { api.runtime.sendMessage({ event: 'record_changed' }).catch(() => {}); },
  random_bytes: size => crypto.getRandomValues(new Uint8Array(size)),
  session_get: async key => (await api.storage.session.get(key))[key] ?? null,
  session_set: async (key, value) => api.storage.session.set({ [key]: value }),
  session_remove: async keys => api.storage.session.remove(keys),
  session_all: async () => api.storage.session.get(null),
  new_tab: async () => { const tab = await api.tabs.create({ url: 'about:blank' }); if (tab.id === undefined) throw new Error('Missing tab ID'); return tab.id; },
  navigate_tab: async (tabId, url) => { await api.tabs.update(tabId, { url }); },
  show_in_tab: (tabId, context) => api.tabs.sendMessage(tabId, { op: 'show_inline', context }),
  active_context: async () => {
    const [tab] = await api.tabs.query({ active: true, currentWindow: true });
    if (!tab || tab.id === undefined) return null;
    let context: {url:string;notices:unknown[];tab_id:number} = { url: tab.url ?? '', notices: [], tab_id: tab.id };
    try { context = { ...context, ...await api.tabs.sendMessage<{url:string;notices:unknown[]}>(tab.id, { op: 'snapshot' }) }; } catch { /* Unsupported or not yet loaded. */ }
    return context;
  },
  schedule_abort: (controller, ms) => setTimeout(() => controller.abort(), ms),
  cancel_timer: id => clearTimeout(id),
};
