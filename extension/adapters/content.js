// Observe a bounded set of X notice elements. Rust interprets their contents.
(() => {
  const api = globalThis.browser ?? chrome;
  const send = message => api.runtime.sendMessage({ ...message, page_url: location.href });
  let timer, generation = 0, lastUrl = location.href;
  let styleReady, themeSheet, themeSignature, host, root, mountedUrl, revision = 0, suppressedRoute, inspectedKey, booting = true;
  const hiddenNotices = new Map();
  const restoreNotices = () => {
    for (const [node, original] of hiddenNotices) {
      node.hidden = original.hidden;
      if (!original.classed) node.classList.remove('limitlessx-notice-hidden');
    }
    hiddenNotices.clear();
  };
  const clearInline = () => {
    ++revision; themeSheet = undefined; themeSignature = undefined; restoreNotices(); host?.remove(); host = undefined; root = undefined; mountedUrl = undefined;
  };
  // Generic DOM transport: Rust supplies the complete view and each action.
  const tags = new Set(['main','header','div','section','article','p','h1','h2','strong','span','time','img','video','a','button','svg','path','br']);
  const attributes = new Set(['class','href','src','alt','loading','crossorigin','referrerpolicy','poster','controls','preload','type','role','target','rel','viewBox','fill','d','aria-label']);
  const element = description => {
    if (typeof description === 'string') return document.createTextNode(description);
    if (!description || !tags.has(description.tag)) throw new Error('Unsupported DOM element');
    const node = ['svg','path'].includes(description.tag)
      ? document.createElementNS('http://www.w3.org/2000/svg', description.tag)
      : document.createElement(description.tag);
    for (const [name, value] of description.attrs ?? []) {
      if (!attributes.has(name)) throw new Error('Unsupported DOM attribute');
      if (['href','src','poster'].includes(name) && new URL(value).protocol !== 'https:') throw new Error('Unsupported resource scheme');
      node.setAttribute(name, value);
    }
    for (const child of description.children ?? []) node.append(element(child));
    if (description.action) node.addEventListener('click', event => {
      if (!event.isTrusted) return;
      node.disabled = true; run(description.action);
    });
    return node;
  };
  const apply = async data => {
    if (!data) return;
    if (data.view && root) root.replaceChildren(element(data.view));
    else if (data.message && root) {
      root.replaceChildren(document.createTextNode(data.message));
      const retry = document.createElement('button'); retry.type = 'button'; retry.textContent = 'Try again';
      retry.addEventListener('click', event => { if (event.isTrusted) { if (data.retry) run(data.retry); else { inspectedKey = undefined; clearInline(); schedule(); } } });
      root.append(retry);
    }
    if (data.load) await run(data.load);
  };
  const run = async message => {
    const attempt = ++revision, sourceUrl = location.href;
    root?.setAttribute('aria-busy', 'true');
    try {
      const response = await send(message);
      if (attempt !== revision || mountedUrl !== sourceUrl || location.href !== sourceUrl || !host?.isConnected) return;
      if (!response?.ok) restoreNotices();
      await apply(response?.ok ? response.data : { ...response, retry: message });
    } catch (error) {
      console.warn('LimitlessX DOM rendering failed:', error);
      if (attempt === revision && root) await apply({ message: 'Could not reach LimitlessX. Try again.', retry: message });
    } finally {
      if (attempt === revision) root?.setAttribute('aria-busy', 'false');
    }
  };
  const syncTheme = () => {
      const background = [document.querySelector('[data-testid="primaryColumn"]'), document.body, document.documentElement]
        .filter(Boolean).map(node => getComputedStyle(node).backgroundColor).find(color => color !== 'rgba(0, 0, 0, 0)' && color !== 'transparent');
      const dark = background ? background.match(/\d+/g)?.slice(0, 3).reduce((a, b) => a + Number(b), 0) < 384 : matchMedia('(prefers-color-scheme: dark)').matches;
      if (!themeSheet || themeSignature === background) return;
      themeSignature = background;
      themeSheet.replaceSync(`:host{--bg:${background ?? (dark ? '#000' : '#fff')};--fg:${dark ? '#e7e9ea' : '#0f1419'};--muted:${dark ? '#71767b' : '#536471'};--line:${dark ? '#2f3336' : '#eff3f4'};--panel:${dark ? '#16181c' : '#f7f9f9'}}`);
  };
  const present = async context => {
    const sourceUrl = location.href;
    try {
      styleReady ??= fetch(api.runtime.getURL('styles/app.css')).then(response => response.text()).then(css => {
        const sheet = new CSSStyleSheet(); sheet.replaceSync(css); return sheet;
      });
      const sheet = await styleReady;
      if (location.href !== sourceUrl) return { ok: false };
      clearInline(); host = document.createElement('div'); host.id = 'limitlessx-inline';
      host.setAttribute('role', 'region'); host.setAttribute('aria-label', 'LimitlessX public content');
      const shadow = host.attachShadow({ mode: 'open' }); shadow.adoptedStyleSheets = [sheet];
      root = document.createElement('div'); root.className = 'inline-root'; shadow.append(root);
      (document.querySelector('[data-testid="primaryColumn"]') ?? document.querySelector('main') ?? document.body).append(host);
      mountedUrl = sourceUrl;
      for (const node of roots()) { hiddenNotices.set(node, { hidden: node.hidden, classed: node.classList.contains('limitlessx-notice-hidden') }); node.classList.add('limitlessx-notice-hidden'); node.hidden = true; }
      themeSheet = new CSSStyleSheet(); syncTheme();
      shadow.adoptedStyleSheets = [sheet, themeSheet];
      await apply(context);
      return { ok: true };
    } catch (error) { console.warn('LimitlessX could not mount its DOM view:', error); clearInline(); styleReady = undefined; return { ok: false }; }
  };
  const roots = () => [...document.querySelectorAll('[data-testid="primaryColumn"] [data-testid="emptyState"], [data-testid="primaryColumn"] [data-testid="tweetTombstone"]')].slice(0, 8);
  const signature = node => (node.textContent ?? '').slice(0, 1600);
  let previousNodes = new Map(roots().map(node => [node, signature(node)]));
  const staleNodes = new Map();
  const snapshot = () => {
    const nodes = roots();
    if (location.href !== lastUrl) {
      for (const [node, text] of previousNodes) if (!staleNodes.has(node)) staleNodes.set(node, { text, url: lastUrl });
      clearInline(); suppressedRoute = undefined; inspectedKey = undefined; lastUrl = location.href; ++generation;
    }
    if (mountedUrl && (!host?.isConnected || mountedUrl !== location.href)) clearInline();
    previousNodes = new Map(nodes.map(node => [node, signature(node)]));
    return {
      url: location.href,
      notices: nodes.filter(node => {
        const stale = staleNodes.get(node);
        if (!stale) return true;
        if (stale.url === location.href || stale.text !== signature(node)) { staleNodes.delete(node); return true; }
        return false;
      }).map(node => ({ text: signature(node), policy_url: node.querySelector('a[href*="post-withheld-by-country"]')?.href ?? '' })),
    };
  };
  const inspect = async () => {
    const context = snapshot(), url = location.href;
    if (booting || suppressedRoute === url || !context.notices.length) return;
    const key = url + JSON.stringify(context.notices);
    if (key === inspectedKey) return;
    inspectedKey = key;
    const attempt = ++generation;
    try {
      const response = await send({ op: 'inspect', notices: context.notices });
      if (attempt !== generation || url !== location.href || !response?.ok || !response.data.eligible) return;
      const opened = await send({ op: 'auto_detected', notices: context.notices });
      if (attempt !== generation || url !== location.href) return;
      if (opened?.ok) await present(opened.data);
    } catch { inspectedKey = undefined; }
  };
  const schedule = () => { clearTimeout(timer); timer = setTimeout(() => { syncTheme(); inspect(); }, 250); };
  const observer = new MutationObserver(schedule);
  observer.observe(document.body, { childList: true, characterData: true, subtree: true });
  const themeObserver = new MutationObserver(schedule);
  themeObserver.observe(document.body, { attributes: true, attributeFilter: ['style', 'class'] });
  themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ['style', 'class'] });
  matchMedia('(prefers-color-scheme: dark)').addEventListener('change', schedule);
  addEventListener('popstate', schedule); addEventListener('pageshow', schedule);
  api.runtime.onMessage.addListener((message, sender, respond) => {
    if (sender.id !== api.runtime.id) return;
    if (message?.op === 'snapshot') respond(snapshot());
    if (message?.op === 'show_inline') {
      suppressedRoute = location.href; present(message.context).then(respond); return true;
    }
  });
  send({ op: 'pending' }).then(response => {
    if (response?.ok && response.data) { suppressedRoute = location.href; return present(response.data); }
  }).catch(() => {}).finally(() => { booting = false; schedule(); });
})();
