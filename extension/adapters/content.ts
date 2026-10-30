// Observe a bounded set of X notice elements. Rust interprets their contents.
(() => {
  const api = globalThis.browser ?? chrome;
  const send = (message: Action) => api.runtime.sendMessage({ ...message, page_url: location.href });
  let timer: ReturnType<typeof setTimeout> | undefined, generation = 0, lastUrl = location.href;
  let styleReady: Promise<CSSStyleSheet> | undefined, themeSheet: CSSStyleSheet | undefined, themeSignature: string | undefined;
  let host: HTMLDivElement | undefined, root: HTMLDivElement | undefined, mountedUrl: string | undefined;
  let revision = 0, suppressedRoute: string | undefined, inspectedKey: string | undefined, booting = true;
  let cachedPresentation: Presentation | undefined, deferredPresentation: {url:string;context:Presentation} | undefined, replacingNative = false;
  let ownedMetadata: {originalTitle:string;title:string;description:string;nodes:[HTMLMetaElement,string|null,string][]} | undefined;
  const restoreMetadata = () => {
    if (!ownedMetadata) return;
    if (document.title === ownedMetadata.title) document.title = ownedMetadata.originalTitle;
    for (const [node,original,written] of ownedMetadata.nodes) if (node.content === written) { if (original === null) node.remove(); else node.content = original; }
    ownedMetadata = undefined;
  };
  const setMetadata = (view: Element) => {
    const title = view.getAttribute('data-page-title');
    if (!title) { restoreMetadata(); return; }
    const description = view.getAttribute('data-page-description') ?? '';
    if (ownedMetadata?.title === title && ownedMetadata.description === description && ownedMetadata.nodes.every(([node]) => node.isConnected)) {
      if (document.title !== title) document.title = title;
      for (const [node,,written] of ownedMetadata.nodes) if (node.content !== written) node.content = written;
      return;
    }
    restoreMetadata();
    ownedMetadata = { originalTitle: document.title, title, description, nodes: [] };
    document.title = title;
    for (const selector of ['meta[property="og:title"]','meta[property="og:description"]','meta[name="description"]']) {
      let node = document.querySelector<HTMLMetaElement>(selector);
      const original = node?.content ?? null;
      if (!node) { node = document.createElement('meta'); const match = selector.match(/\[(property|name)="([^"]+)"\]/)!; node.setAttribute(match[1],match[2]); document.head.append(node); }
      const written = selector.includes('title') ? title : description; ownedMetadata.nodes.push([node,original,written]); node.content = written;
    }
  };
  const hiddenNotices = new Map<HTMLElement,{hidden:boolean;classed:boolean}>();
  const descriptions = new WeakMap<Node,string>(), scrollActions = new WeakMap<Element,Action>();
  let scrollObserver: IntersectionObserver | undefined, scrollArmed = false, lastAutoScroll = -1, autoBusy = false;
  const restoreNotices = () => {
    for (const [node, original] of hiddenNotices) {
      node.hidden = original.hidden;
      if (!original.classed) node.classList.remove('limitlessx-notice-hidden');
    }
    hiddenNotices.clear();
  };
  const clearInline = () => {
    if (root) pauseMedia(root);
    restoreMetadata(); scrollObserver?.disconnect(); scrollArmed = false; lastAutoScroll = -1; autoBusy = false;
    ++revision; themeSheet = undefined; themeSignature = undefined; restoreNotices(); host?.remove(); host = undefined; root = undefined; mountedUrl = undefined;
  };
  // Generic DOM transport: Rust supplies the complete view and each action.
  const tags = new Set(['main','header','div','section','article','p','h1','h2','strong','span','time','img','video','a','button','svg','path','br','input','select','option']);
  const attributes = new Set(['class','href','src','alt','loading','crossorigin','referrerpolicy','poster','controls','preload','type','role','target','rel','viewBox','fill','d','aria-label','aria-hidden','tabindex','title','datetime','data-autoload','data-post-id','playsinline','data-page-title','data-page-description','data-media-role','data-media-command','min','max','step','value','selected']);
  const element = (description: DomDescription): Node => {
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
    if (node.tagName === 'A') node.addEventListener('click', event => event.stopPropagation());
    descriptions.set(node, JSON.stringify(description));
    if (description.action) {
      if (node.hasAttribute('data-autoload')) scrollActions.set(node, description.action);
      const activate = (event: Event) => { if (!event.isTrusted) return; event.stopPropagation(); if (node instanceof HTMLButtonElement) node.disabled = true; run(description.action!); };
      node.addEventListener('click', activate);
      node.addEventListener('keydown', event => { if (event instanceof KeyboardEvent && event.key === 'Enter') { event.preventDefault(); activate(event); } });
    }
    return node;
  };
  const apply = async (data: Presentation | null | undefined): Promise<void> => {
    if (!data) return;
    if (data.view && root) {
      cachedPresentation = data;
      const next = element(data.view), current = root.firstElementChild;
      if (!(next instanceof Element)) throw new Error('Expected an element view');
      setMetadata(next);
      if (current?.matches('.x-reader') && next.matches('.x-reader')) {
        for (const name of ['data-page-title','data-page-description']) { if (next.hasAttribute(name)) current.setAttribute(name,next.getAttribute(name)!); else current.removeAttribute(name); }
        const oldPosts = new Map([...current.children].filter(n => n.hasAttribute('data-post-id')).map(n => [n.getAttribute('data-post-id'), n]));
        const desired = [...next.children].map((n, index) => {
          const old = n.hasAttribute('data-post-id') ? oldPosts.get(n.getAttribute('data-post-id')) : current.children[index];
          return old && !old.matches(':disabled') && !old.querySelector(':disabled') && descriptions.get(old) === descriptions.get(n) ? old : n;
        });
        const keep = new Set(desired);
        for (const child of [...current.childNodes]) if (!(child instanceof Element) || !keep.has(child)) { if (child instanceof Element) pauseMedia(child); child.remove(); }
        desired.forEach((child, index) => { if (current.children[index] !== child) current.insertBefore(child, current.children[index] ?? null); });
      } else { pauseMedia(root); root.replaceChildren(next); }
      enhanceMedia(root);
      observeScroll();
    }
    else if (data.message && root) {
      root.replaceChildren(document.createTextNode(data.message));
      const retry = document.createElement('button'); retry.type = 'button'; retry.textContent = 'Try again';
      retry.addEventListener('click', event => { if (event.isTrusted) { if (data.retry) run(data.retry); else { inspectedKey = undefined; clearInline(); schedule(); } } });
      root.append(retry);
    }
    if (data.load) await run(data.load);
  };
  const checkScroll = () => {
    if (!scrollArmed || autoBusy || !host?.isConnected || Math.abs(scrollY - lastAutoScroll) < 2) return;
    const sentinel = root?.querySelector<HTMLButtonElement>('[data-autoload]');
    if (!sentinel || sentinel.disabled) return;
    const box = sentinel.getBoundingClientRect();
    if (box.top > innerHeight + 240 || box.bottom < 0) return;
    const action = scrollActions.get(sentinel);
    if (!action) return;
    lastAutoScroll = scrollY; autoBusy = true; sentinel.disabled = true;
    sentinel.setAttribute('aria-busy', 'true');
    run(action).finally(() => { autoBusy = false; });
  };
  const observeScroll = () => {
    scrollObserver?.disconnect();
    scrollObserver = new IntersectionObserver(() => checkScroll(), { rootMargin: '240px' });
    const sentinel = root?.querySelector<HTMLButtonElement>('[data-autoload]');
    if (sentinel) scrollObserver.observe(sentinel);
  };
  // A short feed may not yet make the document scrollable. Downward intent still loads its next batch.
  addEventListener('wheel', event => {
    if (!event.isTrusted || event.deltaY <= 0) return;
    scrollArmed = true; lastAutoScroll = -Infinity; requestAnimationFrame(checkScroll);
  }, { passive: true });
  addEventListener('keydown', event => {
    if (!event.isTrusted || !['End','PageDown','ArrowDown',' '].includes(event.key) || (event.target instanceof Element && event.target.closest('input,textarea,[contenteditable=true]'))) return;
    scrollArmed = true; lastAutoScroll = -Infinity; requestAnimationFrame(checkScroll);
  });
  addEventListener('scroll', event => {
    if (!event.isTrusted) return;
    requestAnimationFrame(checkScroll);
  }, { passive: true });
  const run = async (message: Action): Promise<void> => {
    const attempt = ++revision, sourceUrl = location.href;
    root?.setAttribute('aria-busy', 'true');
    try {
      const response = await send(message);
      if (attempt !== revision || mountedUrl !== sourceUrl || location.href !== sourceUrl || !host?.isConnected) return;
      if (!response?.ok && !response?.view) { replacingNative = false; restoreNotices(); }
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
        .filter((node): node is HTMLElement => node instanceof HTMLElement).map(node => getComputedStyle(node).backgroundColor).find(color => color !== 'rgba(0, 0, 0, 0)' && color !== 'transparent');
      const dark = background ? (background.match(/\d+/g)?.slice(0, 3).reduce((a, b) => a + Number(b), 0) ?? 0) < 384 : matchMedia('(prefers-color-scheme: dark)').matches;
      if (!themeSheet || themeSignature === background) return;
      themeSignature = background;
      themeSheet.replaceSync(`:host{--bg:${background ?? (dark ? '#000' : '#fff')};--fg:${dark ? '#e7e9ea' : '#0f1419'};--muted:${dark ? '#71767b' : '#536471'};--line:${dark ? '#2f3336' : '#eff3f4'};--panel:${dark ? '#16181c' : '#f7f9f9'}}`);
  };
  // X owns and can replace the primary column at any time. Never mount elsewhere.
  const reconcilePlacement = () => {
    const primary = document.querySelector('[data-testid="primaryColumn"]');
    if (!primary || !host || mountedUrl !== location.href) return;
    if (host.parentElement !== primary) primary.append(host);
    if (!replacingNative) return;
    const regions = new Set(roots().map(notice => {
      let region = notice;
      for (let ancestor = notice.parentElement; ancestor && ancestor !== primary; ancestor = ancestor.parentElement) {
        if (ancestor.querySelector('[data-testid="UserName"]') && !ancestor.querySelector('h2[role="heading"]')) region = ancestor;
      }
      return region;
    }));
    for (const timeline of primary?.querySelectorAll<HTMLElement>('[aria-label^="Timeline:"]') ?? []) regions.add(timeline);
    for (const node of hiddenNotices.keys()) if (!node.isConnected) hiddenNotices.delete(node);
    for (const node of regions) {
      if (node.contains(host)) continue;
      if (hiddenNotices.has(node) && node.hidden && node.classList.contains('limitlessx-notice-hidden')) continue;
      if (!hiddenNotices.has(node)) hiddenNotices.set(node, { hidden: node.hidden, classed: node.classList.contains('limitlessx-notice-hidden') });
      node.classList.add('limitlessx-notice-hidden'); node.hidden = true;
    }
  };
  const present = async (context: Presentation): Promise<{ok:boolean}> => {
    const sourceUrl = location.href;
    if (!document.querySelector('[data-testid="primaryColumn"]')) {
      deferredPresentation = { url: sourceUrl, context };
      return { ok: true };
    }
    deferredPresentation = undefined;
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
      const primary = document.querySelector('[data-testid="primaryColumn"]');
      if (!primary) { deferredPresentation = { url: sourceUrl, context }; host = undefined; root = undefined; return { ok: true }; }
      primary.append(host);
      mountedUrl = sourceUrl;
      replacingNative = true; reconcilePlacement();
      themeSheet = new CSSStyleSheet(); syncTheme();
      shadow.adoptedStyleSheets = [sheet, themeSheet];
      await apply(context);
      return { ok: true };
    } catch (error) { console.warn('LimitlessX could not mount its DOM view:', error); clearInline(); styleReady = undefined; return { ok: false }; }
  };
  const roots = () => [...document.querySelectorAll<HTMLElement>('[data-testid="primaryColumn"] [data-testid="emptyState"], [data-testid="primaryColumn"] [data-testid="tweetTombstone"]')].slice(0, 8);
  const signature = (node: HTMLElement) => (node.textContent ?? '').slice(0, 1600);
  let previousNodes = new Map(roots().map(node => [node, signature(node)]));
  const staleNodes = new Map<HTMLElement,{text:string;url:string}>();
  const snapshot = () => {
    const nodes = roots();
    if (location.href !== lastUrl) {
      for (const [node, text] of previousNodes) if (!staleNodes.has(node)) staleNodes.set(node, { text, url: lastUrl });
      clearInline(); cachedPresentation = undefined; deferredPresentation = undefined; suppressedRoute = undefined; inspectedKey = undefined; lastUrl = location.href; ++generation; recoverPending();
    }
    if (deferredPresentation?.url === location.href && document.querySelector('[data-testid="primaryColumn"]')) {
      const deferred = deferredPresentation; deferredPresentation = undefined; present(deferred.context);
    }
    if (mountedUrl && !host?.isConnected && mountedUrl === location.href && cachedPresentation) {
      const cached = cachedPresentation; clearInline(); present(cached);
    } else if (mountedUrl && mountedUrl !== location.href) clearInline();
    reconcilePlacement();
    if (mountedUrl === location.href && root?.firstElementChild) setMetadata(root.firstElementChild);
    previousNodes = new Map(nodes.map(node => [node, signature(node)]));
    return {
      url: location.href,
      notices: nodes.filter(node => {
        const stale = staleNodes.get(node);
        if (!stale) return true;
        if (stale.url === location.href || stale.text !== signature(node)) { staleNodes.delete(node); return true; }
        return false;
      }).map(node => ({ text: signature(node), policy_url: node.querySelector<HTMLAnchorElement>('a[href*="post-withheld-by-country"]')?.href ?? '' })),
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
      if (attempt !== generation || url !== location.href || !response?.ok || !response.data?.eligible) return;
      const opened = await send({ op: 'auto_detected', notices: context.notices });
      if (attempt !== generation || url !== location.href) return;
      if (opened?.ok && opened.data) await present(opened.data);
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
    if (message?.op === 'show_inline' && message.context) {
      suppressedRoute = location.href; present(message.context).then(respond); return true;
    }
  });
  const recoverPending = () => {
    const url = location.href, attempt = generation;
    return send({ op: 'pending' }).then(response => {
      if (url === location.href && attempt === generation && response?.ok && response.data) {
        suppressedRoute = url; return present(response.data);
      }
    }).catch(() => {});
  };
  recoverPending().finally(() => { booting = false; schedule(); });
})();
