"""Exercise actual X hydration ordering rather than a ready-at-load fixture."""
import pathlib,tempfile,json
from playwright.sync_api import sync_playwright
from fixtures import POST
ROOT=pathlib.Path(__file__).resolve().parents[2]
with sync_playwright() as p,tempfile.TemporaryDirectory(prefix='limitlessx-lifecycle-') as profile:
 ext=ROOT/'dist/chromium'
 with p.chromium.launch_persistent_context(profile,channel='chromium',headless=True,args=[f'--disable-extensions-except={ext}',f'--load-extension={ext}','--no-sandbox']) as ctx:
  calls=[]
  def provider(route):
   calls.append(route.request.url)
   route.fulfill(json={'code':200,'status':POST,'thread':None,'replies':[],'cursor':{'bottom':None}})
  ctx.route('https://api.fxtwitter.com/**',provider)
  ctx.route('https://x.com/**',lambda r:r.fulfill(body='<html><body><main id="shell"></main></body></html>',content_type='text/html'))
  worker=ctx.service_workers[0] if ctx.service_workers else ctx.wait_for_event('serviceworker');origin='/'.join(worker.url.split('/')[:3]);ui=ctx.new_page();ui.goto(origin+'/popup.html');ui.get_by_role('heading',name='LimitlessX',exact=True).wait_for()
  before=set(ctx.pages);assert ui.evaluate('chrome.runtime.sendMessage({op:"open_manual",input:"https://x.com/jack/status/20"})')['ok']
  x=next(z for z in ctx.pages if z not in before);x.wait_for_load_state();x.wait_for_timeout(600)
  assert x.locator('#limitlessx-inline').count()==0,'reader escaped into body before primary column existed'
  primary='<section data-testid="primaryColumn" style="width:600px"><header><h2 role="heading">Post</h2></header><div aria-label="Timeline: Conversation"><p>Native conversation</p></div></section>'
  x.locator('#shell').evaluate('(n,html)=>n.innerHTML=html',primary)
  x.locator('[data-testid="primaryColumn"] #limitlessx-inline .post-text').wait_for()
  assert x.locator('[aria-label="Timeline: Conversation"]').evaluate('n=>n.hidden')
  assert len(calls)==1
  # X replaces its entire primary column, preserving the surrounding shell.
  for _ in range(2):
   x.locator('#shell').evaluate('(n,html)=>n.innerHTML=html',primary)
   x.locator('[data-testid="primaryColumn"] #limitlessx-inline .post-text').wait_for()
   assert x.locator('[aria-label="Timeline: Conversation"]').evaluate('n=>n.hidden')
  # Native timeline may arrive after the replacement column itself.
  x.locator('[aria-label="Timeline: Conversation"]').evaluate('n=>n.remove()')
  x.locator('[data-testid="primaryColumn"]').evaluate('n=>n.insertAdjacentHTML("afterbegin",\'<div aria-label="Timeline: Conversation">Late native conversation</div>\')')
  x.wait_for_function('document.querySelector(\'[aria-label="Timeline: Conversation"]\').hidden')
  x.wait_for_timeout(600);assert len(calls)==1,'hydration caused repeat provider fetches'
  assert x.locator('#limitlessx-inline').count()==1
  assert x.locator('[data-testid="primaryColumn"] h2').is_visible()
  box=x.locator('#limitlessx-inline').bounding_box();col=x.locator('[data-testid="primaryColumn"]').bounding_box();assert box['width']<=col['width'] and box['x']==col['x']
  print(json.dumps({'deferred_primary':'passed','primary_replacement':'passed','late_native_timeline':'passed','no_refetch':'passed','column_boundaries':'passed'}))
