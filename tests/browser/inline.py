"""Automatic native-looking X DOM rendering, with only an extension popup."""
import json,pathlib,tempfile
from playwright.sync_api import sync_playwright
from fixtures import POST,NOTICE,html
ROOT=pathlib.Path(__file__).resolve().parents[2]
with sync_playwright() as p,tempfile.TemporaryDirectory(prefix='limitlessx-auto-') as profile:
 ext=ROOT/'dist/chromium'
 with p.chromium.launch_persistent_context(profile,channel='chromium',headless=True,args=[f'--disable-extensions-except={ext}',f'--load-extension={ext}','--no-sandbox']) as ctx:
  calls=[]
  def provider(route):calls.append(route.request.url);route.fulfill(json={'code':200,'status':POST})
  ctx.route('https://api.fxtwitter.com/**',provider)
  ctx.route('https://x.com/**',lambda route:route.fulfill(body=html('Account suspended'),content_type='text/html',headers={'Content-Security-Policy':"default-src 'none'; script-src 'self'; style-src 'self'; img-src https://pbs.twimg.com; connect-src 'self'"}))
  worker=ctx.service_workers[0] if ctx.service_workers else ctx.wait_for_event('serviceworker');origin='/'.join(worker.url.split('/')[:3]);ui=ctx.new_page();ui.goto(origin+'/popup.html');ui.get_by_role('heading',name='LimitlessX',exact=True).wait_for();send=lambda m:ui.evaluate('(m)=>chrome.runtime.sendMessage(m)',m)
  send({'op':'clear'});x=ctx.new_page();x.goto('https://x.com/jack/status/20');x.wait_for_timeout(600);assert not calls,'generic error triggered a lookup'
  x.locator('[data-testid="emptyState"]').evaluate("n=>n.style.display='flex'")
  count=len(ctx.pages);x.locator('[data-testid="emptyState"] span').evaluate('(n,text)=>n.textContent=text',NOTICE)
  x.locator('#limitlessx-inline .post-text').wait_for(timeout=10000)
  assert len(ctx.pages)==count;assert x.url=='https://x.com/jack/status/20';assert x.locator('iframe').count()==0
  assert x.get_by_role('button',name='View with LimitlessX').count()==0
  assert x.locator('#limitlessx-inline .post-text').inner_text()==POST['text']
  assert x.locator('[data-testid="emptyState"]').is_hidden(),'withheld notice was not replaced visually'
  assert x.locator('#limitlessx-inline .post-card').evaluate('(n)=>getComputedStyle(n).borderRadius')=='0px'
  assert x.locator('#limitlessx-inline .post-text').evaluate('(n)=>getComputedStyle(n).fontSize')=='20px'
  x.wait_for_timeout(700);assert len(calls)==1,'automatic lookup repeated after its own DOM update'
  x.evaluate("document.body.style.backgroundColor='#15202b'");x.wait_for_timeout(400)
  assert x.locator('#limitlessx-inline .x-reader').evaluate('(n)=>getComputedStyle(n).backgroundColor')=='rgb(21, 32, 43)'
  assert len(calls)==1,'theme change refetched content'
  assert send({'op':'record'})['data']['posts']==0,'automatic display counted as deliberate'
  x.bring_to_front();assert send({'op':'open_current'})['ok'];assert send({'op':'record'})['data']['posts']==1
  ui.get_by_role('button',name='Settings',exact=True).click();ui.get_by_label('Keep a private record').uncheck();assert send({'op':'record'})['data']['enabled'] is False
  ui.get_by_role('button',name='Clear record',exact=True).click();assert send({'op':'record'})['data']['posts']==0
  art=ROOT/'tests/browser/artifacts';art.mkdir(exist_ok=True);ui.set_viewport_size({'width':384,'height':800});ui.screenshot(path=str(art/'popup-settings.png'),full_page=True);x.screenshot(path=str(art/'automatic-native-x.png'),full_page=True)
  x.evaluate("history.pushState({},'', '/jack/status/21');document.body.append(document.createElement('div'))");x.locator('#limitlessx-inline').wait_for(state='detached')
  manifest=json.loads((ext/'manifest.json').read_text());assert 'options_ui' not in manifest;assert {p.name for p in ext.glob('*.html')}=={'popup.html'}
  print(json.dumps({'automatic_injection':'passed','generic_failure_ignored':'passed','native_timeline_style':'passed','single_automatic_request':'passed','no_reader_page':'passed','popup_settings':'passed','navigation_cleanup':'passed','automatic_record_count':0,'deliberate_record_count':1}))
