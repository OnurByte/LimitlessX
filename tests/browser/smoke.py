"""Real extension smoke test; provider-independent and no X credentials."""
import json, pathlib, tempfile, sys
from playwright.sync_api import sync_playwright
ROOT=pathlib.Path(__file__).resolve().parents[2]
with sync_playwright() as p, tempfile.TemporaryDirectory(prefix='limitlessx-chromium-') as profile:
 extension=ROOT/'dist/chromium'
 with p.chromium.launch_persistent_context(profile,channel="chromium",headless=True,args=[f'--disable-extensions-except={extension}',f'--load-extension={extension}','--no-sandbox']) as context:
  worker=context.service_workers[0] if context.service_workers else context.wait_for_event('serviceworker')
  origin='/'.join(worker.url.split('/')[:3]);page=context.new_page();page.goto(origin+'/popup.html')
  page.get_by_role('heading',name='LimitlessX',exact=True).wait_for()
  response=page.evaluate('chrome.runtime.sendMessage({op:"ping"})')
  assert response=={'ok':True,'data':{'runtime':'rust-wasm'}},response
  print(json.dumps({'browser':'chromium','runtime':response,'popup':page.title()}))
