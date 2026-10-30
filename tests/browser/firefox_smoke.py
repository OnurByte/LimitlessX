from paths import firefox_binary,geckodriver
"""Install the actual Firefox MV3 package through WebDriver and exercise WASM."""
import json,pathlib,tempfile,zipfile,os
from selenium import webdriver
from selenium.webdriver.firefox.options import Options
from selenium.webdriver.firefox.service import Service
from selenium.webdriver.support.ui import WebDriverWait
ROOT=pathlib.Path(__file__).resolve().parents[2]
opts=Options();opts.add_argument('-headless')
opts.binary_location=firefox_binary()
opts.set_preference('xpinstall.signatures.required',False)
opts.set_preference('extensions.webextensions.uuids',json.dumps({'limitlessx@onurbyte.dev':'9bc48fdd-5c45-49d7-a397-b86b3402f001'}))
with tempfile.TemporaryDirectory(prefix='limitlessx-firefox-') as folder:
 addon=pathlib.Path(folder)/'limitlessx.xpi'
 with zipfile.ZipFile(addon,'w',compression=zipfile.ZIP_DEFLATED) as z:
  for p in (ROOT/'dist/firefox').rglob('*'):
   if p.is_file():z.write(p,p.relative_to(ROOT/'dist/firefox'))
 with webdriver.Firefox(options=opts,service=Service(geckodriver(),env={**os.environ,'MOZ_REMOTE_ALLOW_SYSTEM_ACCESS':'1'},log_output=str(pathlib.Path(folder)/'gecko.log'))) as driver:
  driver.install_web_extension(path=str(addon),permanent=False)
  driver.set_context('chrome')
  driver.execute_script("gBrowser.selectedBrowser.loadURI(Services.io.newURI('moz-extension://9bc48fdd-5c45-49d7-a397-b86b3402f001/popup.html'),{triggeringPrincipal:Services.scriptSecurityManager.getSystemPrincipal()});")
  driver.set_context('content')
  WebDriverWait(driver,30).until(lambda d:d.find_elements('tag name','h1'))
  result=driver.execute_async_script('const done=arguments[0];browser.runtime.sendMessage({op:"ping"}).then(done,e=>done({error:String(e)}));')
  assert result=={'ok':True,'data':{'runtime':'rust-wasm'}},result
  print(json.dumps({'browser':'firefox','runtime':result,'popup':driver.title}))
