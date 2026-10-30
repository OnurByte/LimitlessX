"""Real observed failure: username endpoints 404, exact public UID endpoints succeed."""
import copy,json,pathlib,tempfile
from playwright.sync_api import sync_playwright
from fixtures import USER,POST,html
ROOT=pathlib.Path(__file__).resolve().parents[2]
with sync_playwright() as p,tempfile.TemporaryDirectory(prefix='limitlessx-identity-') as profile:
 ext=ROOT/'dist/chromium'
 with p.chromium.launch_persistent_context(profile,channel='chromium',headless=True,args=[f'--disable-extensions-except={ext}',f'--load-extension={ext}','--no-sandbox']) as ctx:
  calls=[];user={**USER,'name':'Public identity','description':'Verified biography','followers':1234,'following':25,'statuses':321};post={**copy.deepcopy(POST),'author':user}
  def provider(route):
   url=route.request.url;calls.append(url)
   if '/typeahead' in url:route.fulfill(json={'code':200,'users':[{'id':'99','screen_name':'jack_fan','protected':False},{**user,'followers':0,'following':0}]})
   elif '/id:12/statuses' in url:route.fulfill(json={'code':200,'results':[post],'cursor':{'bottom':None}})
   elif '/id:12' in url:route.fulfill(json={'code':200,'user':user})
   else:route.fulfill(status=404,json={'code':404,'message':'User not found'})
  ctx.route('https://api.fxtwitter.com/**',provider)
  ctx.route('https://x.com/**',lambda r:r.fulfill(body=html('Account Withheld@jack has been withheld in TR in response to a legal demand.').replace('<head>','<head><title>Account Withheld / X</title><meta property="og:title" content="Account Withheld / X"><meta name="description" content="Original">'),content_type='text/html'))
  worker=ctx.service_workers[0] if ctx.service_workers else ctx.wait_for_event('serviceworker');origin='/'.join(worker.url.split('/')[:3]);ui=ctx.new_page();ui.goto(origin+'/popup.html');ui.get_by_role('heading',name='LimitlessX',exact=True).wait_for()
  x=ctx.new_page();x.goto('https://x.com/jack');x.locator('#limitlessx-inline .post-card').wait_for()
  assert sum('/typeahead'in u for u in calls)==1,'parallel recovery did not share identity resolution'
  assert x.locator('[data-testid="emptyState"]').is_hidden();assert x.get_by_role('alert').count()==0
  assert x.locator('#limitlessx-inline .x-profile-count').all_text_contents()==['25 Following','1.2K Followers'],'placeholder suggestion counters leaked into metadata'
  assert x.title()=='Public identity (@jack) / X'
  assert x.locator('meta[property="og:title"]').get_attribute('content')==x.title()
  assert x.locator('meta[name="description"]').get_attribute('content')=='Verified biography'
  ui.wait_for_function('document.querySelector(".metrics strong")?.textContent==="1"')
  assert ui.evaluate('chrome.runtime.sendMessage({op:"record"})')['data']['profiles']==1
  x.reload();x.locator('#limitlessx-inline .post-card').wait_for();assert ui.evaluate('chrome.runtime.sendMessage({op:"record"})')['data']['profiles']==1
  x.evaluate("history.pushState({},'', '/other');document.body.append(document.createElement('div'))");x.locator('#limitlessx-inline').wait_for(state='detached');assert x.title()=='Account Withheld / X'
  assert x.locator('meta[name="description"]').get_attribute('content')=='Original'
  ui.set_viewport_size({'width':384,'height':800});ui.screenshot(path=str(ROOT/'tests/browser/artifacts/brand-popup.png'),full_page=True)
  print(json.dumps({'username_404_recovery':'passed','exact_uid_only':'passed','shared_resolution':'passed','automatic_unique_counter':'passed','live_popup_counter':'passed','metadata_restore':'passed','suggestion_zero_counts_ignored':'passed'}))
