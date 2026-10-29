import copy,json,pathlib,tempfile,time
from urllib.parse import urlsplit,parse_qs
from playwright.sync_api import sync_playwright
from fixtures import USER,POST,NOTICE,html
ROOT=pathlib.Path(__file__).resolve().parents[2]
report={}
with sync_playwright() as p,tempfile.TemporaryDirectory(prefix='limitlessx-contracts-') as profile:
 extension=ROOT/'dist/chromium'
 def launch():return p.chromium.launch_persistent_context(profile,channel='chromium',headless=True,args=[f'--disable-extensions-except={extension}',f'--load-extension={extension}','--no-sandbox'])
 with launch() as ctx:
  worker=ctx.service_workers[0] if ctx.service_workers else ctx.wait_for_event('serviceworker');origin='/'.join(worker.url.split('/')[:3])
  errors=[];ctx.on('weberror',lambda e:errors.append(str(e.error)))
  ui=ctx.new_page();ui.goto(origin+'/popup.html');ui.get_by_role('heading',name='LimitlessX',exact=True).wait_for()
  def send(m):return ui.evaluate('(m)=>chrome.runtime.sendMessage(m)',m)
  def open_manual(target):
   with ctx.expect_page() as opened:assert send({'op':'open_manual','input':target})['ok']
   reader=opened.value;reader.wait_for_load_state();return reader
  calls=[]; mode={'value':'success'}
  def provider(route):
   calls.append(route.request.url)
   assert not route.request.headers.get('cookie'),'credentials leaked'
   url=route.request.url
   if mode['value']=='429':route.fulfill(status=429,json={'code':429},headers={'Retry-After':'1'});return
   if mode['value']=='private':route.fulfill(json={'code':200,'user':{**USER,'protected':True}});return
   if mode['value']=='malformed':route.fulfill(body='<html>oops</html>');return
   if '/statuses' in url:
    post=copy.deepcopy(POST)
    if 'cursor=' in url:post['id']='21';post['text']='second page'
    route.fulfill(json={'code':200,'results':[post],'cursor':{'bottom':None if 'cursor=' in url else 'next+/='}})
   elif '/profile/' in url:route.fulfill(json={'code':200,'user':USER})
   else:route.fulfill(json={'code':200,'status':POST})
  ctx.route('https://api.fxtwitter.com/**',provider)
  assert not send({'op':'open_manual','input':'https://x.com.evil.test/jack'})['ok'];assert not calls
  send({'op':'clear'});reader=open_manual('@jack');reader.get_by_role('heading',name='jack',exact=True).wait_for()
  assert reader.locator('.biography').inner_text()==USER['description'];assert reader.evaluate('globalThis.injected') is None
  assert send({'op':'record'})['data']['profiles']==0
  reader.get_by_role('button',name='Load more posts').click();reader.get_by_text('second page',exact=True).wait_for();assert reader.locator('.post-card').count()==2
  token=parse_qs(urlsplit(reader.url).query)['session'][0]
  assert not send({'op':'open_post','session':token,'id':'999'})['ok']
  assert send({'op':'record'})['data']['posts']==0
  report['manual_profile_pagination_xss']='passed'
  # Suspended text and user-authored tweets must not produce a restriction action.
  ctx.route('https://x.com/**',lambda r:r.fulfill(body=html('Account suspended'),content_type='text/html'))
  x=ctx.new_page();x.goto('https://x.com/jack/status/20');x.wait_for_timeout(600);assert x.get_by_role('button',name='View with LimitlessX').count()==0
  assert x.evaluate('typeof chrome.runtime?.sendMessage')=='undefined'
  # Install an eligible notice, then navigate without replacing its DOM node.
  x.locator('[data-testid="emptyState"] span').evaluate('(n,text)=>n.textContent=text',NOTICE)
  x.get_by_role('button',name='View with LimitlessX').wait_for()
  x.evaluate("history.pushState({},'', '/jack/status/21');document.body.append(document.createElement('div'))")
  x.wait_for_timeout(600)
  assert x.get_by_role('button',name='View with LimitlessX').count()==0,'stale notice applied to a new route'
  report['stale_notice_navigation']='passed'
  # A new notice node is eligible on the new route.
  x.locator('[data-testid="emptyState"]').evaluate('(n)=>n.replaceWith(n.cloneNode(true))')
  x.get_by_role('button',name='View with LimitlessX').wait_for()
  # Restarts: stop workers, then verify reader session and durable record remain.
  cdp=ctx.new_cdp_session(ui);cdp.send('ServiceWorker.enable');cdp.send('ServiceWorker.stopAllWorkers')
  assert send({'op':'record'})['ok']
  assert send({'op':'timeline','session':token,'target':{'kind':'profile','handle':'jack'},'cursor':None})['ok']
  report['worker_restart']='passed'
  # Expired sessions must be rejected before any network request.
  ui.evaluate('''async token=>{const key='reader:'+token;const data=await chrome.storage.session.get(key);data[key].expires=0;await chrome.storage.session.set(data);}''',token)
  previous=len(calls);assert not send({'op':'read','session':token,'target':{'kind':'profile','handle':'jack'}})['ok'];assert len(calls)==previous
  report['expiry_before_fetch']='passed'
  # Rate-limit cooldown suppresses repeat requests.
  mode['value']='429';r=open_manual('https://x.com/jack/status/20');r.get_by_role('alert').wait_for();previous=len(calls)
  r.get_by_role('button',name='Try again').click();r.get_by_role('alert').wait_for();assert len(calls)==previous
  time.sleep(1.1);mode['value']='malformed';r.get_by_role('button',name='Try again').click();r.get_by_text('FxTwitter returned an unexpected response.',exact=True).wait_for()
  report['rate_limit_and_malformed']='passed'
  send({'op':'settings','enabled':False});saved=send({'op':'record'})['data']
  # Capture actual layouts for visual QA, including dark mode.
  art=ROOT/'tests/browser/artifacts';art.mkdir(exist_ok=True)
  ui.set_viewport_size({'width':384,'height':800});ui.screenshot(path=str(art/'popup-light.png'),full_page=True)
  ui.emulate_media(color_scheme='dark');ui.screenshot(path=str(art/'popup-dark.png'),full_page=True)
  reader.screenshot(path=str(art/'reader.png'),full_page=True)
  assert not errors,errors
 with launch() as ctx:
  ui=ctx.new_page();ui.goto(origin+'/options.html');ui.get_by_role('heading',name='LimitlessX',exact=True).wait_for()
  actual=ui.evaluate('chrome.runtime.sendMessage({op:"record"})')['data'];assert actual==saved,(actual,saved)
  assert ui.evaluate('chrome.storage.session.get(null)')=={},'reader context survived full browser restart'
  # Durable keys must not contain public IDs or handles.
  keys=ui.evaluate('''async()=>{const db=await new Promise((res,rej)=>{const r=indexedDB.open('limitlessx');r.onsuccess=()=>res(r.result);r.onerror=()=>rej(r.error);});return Promise.all(['profiles','posts'].map(name=>new Promise((res,rej)=>{const r=db.transaction(name).objectStore(name).getAllKeys();r.onsuccess=()=>res(r.result);r.onerror=()=>rej(r.error);})));}''')
  assert all(len(k)==64 for store in keys for k in store),keys
  report['browser_restart_and_local_storage']='passed'
print(json.dumps(report,indent=2))
