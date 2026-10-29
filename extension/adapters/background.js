import './bridge.js';
import init, { dispatch } from '../pkg/runtime/limitless_runtime.js';
const api = globalThis.browser ?? chrome;
const ready = init();
// Register synchronously, before any WASM initialization or asynchronous work.
api.runtime.onMessage.addListener((message, sender, respond) => {
  ready.then(() => dispatch(message, {
    id: sender.id ?? '', url: sender.url ?? '', tab_id: sender.tab?.id ?? null,
    frame_id: sender.frameId ?? 0, extension_origin: api.runtime.getURL('')
  })).then(respond, () => respond({ ok: false, error: 'unavailable', message: 'Could not initialize LimitlessX. Reopen the extension.' }));
  return true;
});
