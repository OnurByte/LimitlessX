"""Behavioral contract for the packaged extension, real IDB and real WASM.
Provider transport is intercepted; response normalization and all application logic stay real.
"""
import json,pathlib,tempfile
from urllib.parse import urlsplit,parse_qs
from playwright.sync_api import sync_playwright
ROOT=pathlib.Path(__file__).resolve().parents[2]
USER={'type':'profile','id':'12','screen_name':'jack','name':'jack','description':'<script>bad()</script>','protected':False,'avatar_url':None}
POST={'type':'status','provider':'twitter','id':'20','text':'just setting up my twttr','author':USER,'created_at':None,'media':{}}
NOTICE="This post has been withheld in Türkiye in response to a legal demand."
HTML=f'''<html><body><main><div data-testid="primaryColumn"><div data-testid="emptyState"><span>{NOTICE}</span><a href="https://help.x.com/en/rules-and-policies/post-withheld-by-country">Learn more</a></div></div></main></body></html>'''
with sync_playwright() as p,tempfile.TemporaryDirectory(prefix='limitlessx-flows-') as profile:
 ext=ROOT/'dist/chromium'
 with p.chromium.launch_persistent_context(profile,channel="chromium",headless=True,args=[f'--disable-extensions-except={ext}',f'--load-extension={ext}','--no-sandbox']) as ctx:
  ctx.route('https://api.fxtwitter.com/**',lambda route:route.fulfill(json={'code':200,'status':POST}))
  worker=ctx.service_workers[0] if ctx.service_workers else ctx.wait_for_event('serviceworker');origin='/'.join(worker.url.split('/')[:3])
  ui=ctx.new_page();ui.goto(origin+'/popup.html');ui.get_by_role('heading',name='LimitlessX',exact=True).wait_for()
  def send(message):return ui.evaluate('(m)=>chrome.runtime.sendMessage(m)',message)
  assert send({'op':'clear'})['ok']
  assert send({'op':'record'})['data'].get('posts')==0
  with ctx.expect_page() as opened:send({'op':'open_manual','input':'https://x.com/jack/status/20'})
  reader=opened.value;reader.wait_for_load_state();reader.get_by_text('just setting up my twttr',exact=True).wait_for()
  assert send({'op':'record'})['data']['posts']==0,'manual lookup must not count'
  ctx.route('https://x.com/jack/status/20',lambda route:route.fulfill(body=HTML,content_type='text/html'))
  x=ctx.new_page();x.goto('https://x.com/jack/status/20');
  with ctx.expect_page() as opened:x.get_by_role('button',name='View with LimitlessX').click()
  reader=opened.value;reader.wait_for_load_state();reader.get_by_text('just setting up my twttr',exact=True).wait_for()
  assert send({'op':'record'})['data']['posts']==1
  token=parse_qs(urlsplit(reader.url).query)['session'][0];target={'kind':'post','id':'20'}
  results=ui.evaluate('''async ([session,target])=>Promise.all(Array.from({length:12},()=>chrome.runtime.sendMessage({op:'read',session,target})))''',[token,target]);assert all(r['ok'] for r in results),results
  assert send({'op':'record'})['data']['posts']==1,'duplicate/racing views must not increment'
  assert not send({'op':'read','session':token,'target':{'kind':'post','id':'21'}})['ok'],'target substitution accepted'
  send({'op':'settings','enabled':False});send({'op':'clear'})
  assert send({'op':'read','session':token,'target':target})['ok'];assert send({'op':'record'})['data']['posts']==0
  send({'op':'settings','enabled':True})
  print(json.dumps({'manual_count':0,'concurrent_duplicate_count':1,'disabled_count':0,'target_substitution':'rejected'}))
