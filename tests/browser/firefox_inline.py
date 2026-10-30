from paths import firefox_binary,geckodriver
"""Firefox inline UI test. X DOM is controlled; status/20 uses the real public provider.
Firefox BiDi does not intercept extension event-page Fetch, so this test explicitly
reports a live-provider dependency instead of claiming a deterministic fixture.
"""
import json,pathlib,tempfile,zipfile,os
from selenium import webdriver
from selenium.webdriver.firefox.options import Options
from selenium.webdriver.firefox.service import Service
from selenium.webdriver.support.ui import WebDriverWait
from fixtures import html
ROOT=pathlib.Path(__file__).resolve().parents[2]
opts=Options();opts.add_argument('-headless');opts.enable_bidi=True
opts.binary_location=firefox_binary()
opts.set_preference('xpinstall.signatures.required',False)
opts.set_preference('extensions.webextensions.uuids',json.dumps({'limitlessx@onurbyte.dev':'9bc48fdd-5c45-49d7-a397-b86b3402f001'}))
with tempfile.TemporaryDirectory(prefix='limitlessx-firefox-inline-') as folder:
 addon=pathlib.Path(folder)/'limitlessx.xpi'
 with zipfile.ZipFile(addon,'w',compression=zipfile.ZIP_DEFLATED) as z:
  for path in (ROOT/'dist/firefox').rglob('*'):
   if path.is_file():z.write(path,path.relative_to(ROOT/'dist/firefox'))
 service=Service(geckodriver(),env={**os.environ,'MOZ_REMOTE_ALLOW_SYSTEM_ACCESS':'1'},log_output=str(pathlib.Path(folder)/'gecko.log'))
 with webdriver.Firefox(options=opts,service=service) as d:
  d.install_web_extension(str(addon),permanent=False);wait=WebDriverWait(d,30)
  d.network.add_request_handler(['https://x.com/**'],lambda req:req.provide_response(status=200,headers={'Content-Type':'text/html; charset=utf-8','Content-Security-Policy':"default-src 'none'; script-src 'self'; style-src 'self'; img-src https://pbs.twimg.com; connect-src 'self'"},body=html()))
  d.set_context('chrome');d.execute_script("gBrowser.selectedBrowser.loadURI(Services.io.newURI(arguments[0]),{triggeringPrincipal:Services.scriptSecurityManager.getSystemPrincipal()});",'moz-extension://9bc48fdd-5c45-49d7-a397-b86b3402f001/popup.html');d.set_context('content');wait.until(lambda d:d.find_elements('tag name','h1'));ui=d.current_window_handle
  def send(m):return d.execute_async_script('const [m,done]=arguments;browser.runtime.sendMessage(m).then(done,e=>done({error:String(e)}));',m)
  def post_text(d):
   try:return d.find_element('id','limitlessx-inline').shadow_root.find_element('class name','post-text')
   except Exception:return False
  assert send({'op':'clear'})['ok'];handles=set(d.window_handles)
  assert send({'op':'open_manual','input':'https://x.com/jack/status/20'})['ok'];wait.until(lambda d:len(d.window_handles)>len(handles));manual=next(h for h in d.window_handles if h not in handles);d.switch_to.window(manual)
  assert wait.until(post_text).text=='just setting up my twttr';assert d.current_url.startswith('https://x.com/');assert not d.find_elements('tag name','iframe')
  d.switch_to.window(ui);assert send({'op':'record'})['data']['posts']==0
  d.switch_to.new_window('tab');d.get('https://x.com/jack/status/20');handles=set(d.window_handles);assert wait.until(post_text).text=='just setting up my twttr';assert set(d.window_handles)==handles
  xhandle=d.current_window_handle;d.switch_to.window(ui);assert send({'op':'record'})['data']['posts']==1
  # An explicit popup action is deliberate, while the automatic display stays uncounted.
  d.switch_to.window(xhandle);d.set_context('chrome');d.execute_script('window.focus();');d.set_context('content')
  # sendMessage runs from the popup context, with the X tab selected in the browser.
  d.switch_to.window(ui);assert send({'op':'record'})['data']['posts']==1
  d.find_element('class name','settings-toggle').click();checkbox=wait.until(lambda d:d.find_elements('css selector','input[type=checkbox]'))[0];checkbox.click();wait.until(lambda d:send({'op':'record'})['data']['enabled'] is False)
  d.find_element('class name','record-controls').find_element('tag name','button').click();wait.until(lambda d:send({'op':'record'})['data']['posts']==0)
  print(json.dumps({'browser':'firefox','automatic_injection':'passed','strict_page_csp':'passed','no_extension_reader_page':'passed','manual_count':0,'automatic_count':1,'popup_settings':'passed','provider':'live FxTwitter status/20'}))
