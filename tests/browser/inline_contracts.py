import json,pathlib,tempfile,copy,time
from playwright.sync_api import sync_playwright
from fixtures import USER,POST,NOTICE,html
ROOT=pathlib.Path(__file__).resolve().parents[2]
report={}
with sync_playwright() as p,tempfile.TemporaryDirectory(prefix='limitlessx-inline-contracts-') as profile:
 ext=ROOT/'dist/chromium'
 def launch():return p.chromium.launch_persistent_context(profile,channel='chromium',headless=True,args=[f'--disable-extensions-except={ext}',f'--load-extension={ext}','--no-sandbox'])
 with launch() as ctx:
  worker=ctx.service_workers[0] if ctx.service_workers else ctx.wait_for_event('serviceworker');origin='/'.join(worker.url.split('/')[:3]);ui=ctx.new_page();ui.goto(origin+'/popup.html');ui.get_by_role('heading',name='LimitlessX',exact=True).wait_for()
  send=lambda m:ui.evaluate('(m)=>chrome.runtime.sendMessage(m)',m)
  mode={'value':'success'};calls=[]
  def provider(route):
   calls.append(route.request.url);url=route.request.url
   assert not route.request.headers.get('cookie')
   if mode['value']=='429':route.fulfill(status=429,json={'code':429},headers={'Retry-After':'1'});return
   if mode['value']=='malformed':route.fulfill(body='not JSON');return
   if '/statuses' in url:
    post=copy.deepcopy(POST)
    if 'cursor=' in url:post['id']='21';post['text']='second page'
    route.fulfill(json={'code':200,'results':[post],'cursor':{'bottom':None if 'cursor=' in url else 'next+/='}})
   elif '/profile/' in url:route.fulfill(json={'code':200,'user':USER})
   else:
    post=copy.deepcopy(POST)
    if '/status/21' in url or '/conversation/21' in url:post['id']='21';post['text']='new route post'
    route.fulfill(json={'code':200,'status':post,'thread':None,'replies':[],'cursor':{'bottom':None}})
  ctx.route('https://api.fxtwitter.com/**',provider)
  ctx.route('https://x.com/**',lambda r:r.fulfill(body=html(),content_type='text/html',headers={'Content-Security-Policy':"default-src 'none'; script-src 'self'; style-src 'self'; img-src https://pbs.twimg.com; connect-src 'self'"}))
  send({'op':'clear'})
  assert not send({'op':'open_manual','input':'https://x.com.evil.test/jack'})['ok'];assert not calls
  with ctx.expect_page() as opened:assert send({'op':'open_manual','input':'@jack'})['ok']
  x=opened.value;x.wait_for_load_state();x.locator('#limitlessx-inline .biography').wait_for()
  assert x.url=='https://x.com/jack';assert x.locator('#limitlessx-inline .biography').inner_text()==USER['description'];assert x.evaluate('globalThis.injected') is None
  x.get_by_role('button',name='Load more posts').click();x.get_by_text('second page',exact=True).wait_for();assert x.locator('#limitlessx-inline .post-card').count()==2
  assert send({'op':'record'})['data']['profiles']==0;assert send({'op':'record'})['data']['posts']==0
  report['manual_injected_profile_pagination_xss']='passed'
  # Run messages in the extension's real isolated content-script world.
  def content_sender(page):
   cdp=ctx.new_cdp_session(page);worlds=[];cdp.on('Runtime.executionContextCreated',lambda e:worlds.append(e['context']));cdp.send('Runtime.enable')
   world=next(w for w in worlds if w['name']=='LimitlessX')
   def send_content(message):
    result=cdp.send('Runtime.evaluate',{'contextId':world['id'],'expression':'chrome.runtime.sendMessage({...'+json.dumps(message)+',page_url:location.href})','awaitPromise':True,'returnByValue':True});assert 'exceptionDetails' not in result,result;return result['result']['value']
   return send_content
  isolated=content_sender(x)
  sessions=ui.evaluate('chrome.storage.session.get(null)');token=next(k[7:] for k,v in sessions.items() if v['source']=={'kind':'profile','handle':'jack'})
  assert not isolated({'op':'open_post','session':token,'id':'999'})['ok']
  x.locator('#limitlessx-inline .post-text').first.click();x.locator('#limitlessx-inline .detailed-post .post-text').wait_for();assert x.locator('#limitlessx-inline .post-card').count()==1
  assert len(ctx.pages)==3,'opening a timeline post created another tab'
  assert x.url=='https://x.com/jack';assert send({'op':'record'})['data']['posts']==0
  report['same_container_post_open']='passed'
  # A restricted direct-post flow counts once, even under racing reads.
  z=ctx.new_page();z.goto('https://x.com/jack/status/20');action=z.locator('#limitlessx-inline .post-text').wait_for()
  assert send({'op':'record'})['data']['posts']==1
  z.bring_to_front();assert send({'op':'open_current'})['ok']
  assert send({'op':'record'})['data']['posts']==1
  sessions=ui.evaluate('chrome.storage.session.get(null)');token=next(k[7:] for k,v in sessions.items() if v['restricted'] and v.get('deliberate') and v['target']=={'kind':'post','id':'20'})
  content=content_sender(z);request={'op':'read','session':token,'target':{'kind':'post','id':'20'}}
  assert not isolated(request)['ok'],'session transferred to another X tab'
  assert not send(request)['ok'],'popup accepted content-only reader operation'
  for _ in range(4):assert content(request)['ok']
  assert send({'op':'record'})['data']['posts']==1
  report['tab_binding_and_dedup']='passed'
  cdp=ctx.new_cdp_session(ui);cdp.send('ServiceWorker.enable');cdp.send('ServiceWorker.stopAllWorkers');assert content(request)['ok'];report['worker_restart']='passed'
  ui.evaluate('''async token=>{const key='reader:'+token;const data=await chrome.storage.session.get(key);data[key].expires=0;await chrome.storage.session.set(data);}''',token)
  before=len(calls);assert not content(request)['ok'];assert len(calls)==before;report['expiry_before_fetch']='passed'
  # Reused notices from an old route must never apply to its new target.
  z.evaluate("history.pushState({},'', '/jack/status/21');document.body.append(document.createElement('div'))");z.locator('#limitlessx-inline').wait_for(state='detached');z.wait_for_timeout(400)
  assert z.get_by_role('button',name='View with LimitlessX').count()==0;report['navigation_cleanup']='passed'
  z.locator('[data-testid="emptyState"] span').evaluate('(n,text)=>n.textContent=text',NOTICE.replace('Türkiye','Germany'))
  z.get_by_text('new route post',exact=True).wait_for();report['reused_notice_refresh']='passed'
  mode['value']='429'
  with ctx.expect_page() as opened:send({'op':'open_manual','input':'https://x.com/jack/status/20'})
  limited=opened.value;limited.get_by_role('alert').wait_for();before=len(calls);limited.get_by_role('button',name='Try again').click();limited.get_by_role('alert').wait_for();assert len(calls)==before
  time.sleep(1.1);mode['value']='malformed';limited.get_by_role('button',name='Try again').click();limited.get_by_text('FxTwitter returned an unexpected response.',exact=True).wait_for();report['rate_limit_and_malformed']='passed'
  send({'op':'settings','enabled':False});saved=send({'op':'record'})['data']
  art=ROOT/'tests/browser/artifacts';art.mkdir(exist_ok=True);x.screenshot(path=str(art/'inline-content.png'),full_page=True)
 with launch() as ctx:
  ui=ctx.new_page();ui.goto(origin+'/popup.html');ui.get_by_role('heading',name='LimitlessX',exact=True).wait_for();actual=ui.evaluate('chrome.runtime.sendMessage({op:"record"})')['data'];assert actual==saved;assert ui.evaluate('chrome.storage.session.get(null)')=={};report['browser_restart']='passed'
print(json.dumps(report,indent=2))
