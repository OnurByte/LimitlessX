import init from '../pkg/ui/limitless_ui.js';
const api = globalThis.browser ?? chrome;
globalThis.limitlessUI = {
  send: message => api.runtime.sendMessage(message),
  asset_url: path => api.runtime.getURL(path),
};
api.runtime.onMessage.addListener((message, sender) => {
  if (sender.id === api.runtime.id && message?.event === 'record_changed') dispatchEvent(new Event('focus'));
});
try { await init(); }
catch {
  const main = document.createElement('main');
  main.textContent = 'LimitlessX could not start. Reopen the extension or reinstall the package.';
  document.body.append(main);
}
