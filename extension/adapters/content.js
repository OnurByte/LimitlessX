// Observe a bounded set of X notice elements. Rust interprets their contents.
(() => {
  const api = globalThis.browser ?? chrome;
  let timer, generation = 0, button, lastUrl = location.href;
  const snapshot = () => ({
    url: location.href,
    notices: [...document.querySelectorAll('[data-testid="primaryColumn"] [data-testid="emptyState"], [data-testid="primaryColumn"] [data-testid="tweetTombstone"]')]
      .slice(0, 8).map(node => ({
        text: (node.innerText ?? '').slice(0, 1600),
        policy_url: node.querySelector('a[href*="post-withheld-by-country"]')?.href ?? '',
      })),
  });
  const removeButton = () => { button?.remove(); button = undefined; };
  const inspect = async () => {
    const attempt = ++generation, url = location.href, context = snapshot();
    if (url !== lastUrl) { removeButton(); lastUrl = url; }
    if (!context.notices.length) { removeButton(); return; }
    let response;
    try { response = await api.runtime.sendMessage({ op: 'inspect', notices: context.notices }); } catch { removeButton(); return; }
    if (attempt !== generation || url !== location.href) return;
    if (!response?.ok || !response.data.eligible) { removeButton(); return; }
    if (button?.isConnected) return;
    button = document.createElement('button');
    button.className = 'limitlessx-action'; button.type = 'button'; button.textContent = 'View with LimitlessX';
    button.addEventListener('click', async () => {
      const activeButton = button; if (!activeButton) return;
      activeButton.disabled = true;
      try {
        const result = await api.runtime.sendMessage({ op: 'open_detected', notices: snapshot().notices });
        if (!result?.ok) activeButton.textContent = result?.message ?? 'Could not open LimitlessX. Try again.';
      } catch { activeButton.textContent = 'Could not open LimitlessX. Try again.'; }
      finally { activeButton.disabled = false; }
    });
    document.querySelector('[data-testid="primaryColumn"] [data-testid="emptyState"], [data-testid="primaryColumn"] [data-testid="tweetTombstone"]')?.append(button);
  };
  const schedule = () => { clearTimeout(timer); timer = setTimeout(inspect, 250); };
  const observer = new MutationObserver(schedule);
  observer.observe(document.body, { childList: true, subtree: true });
  addEventListener('popstate', schedule); addEventListener('pageshow', schedule);
  api.runtime.onMessage.addListener((message, sender, respond) => {
    if (sender.id === api.runtime.id && message?.op === 'snapshot') respond(snapshot());
  });
  schedule();
})();
