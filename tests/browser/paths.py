import os,shutil
from playwright.sync_api import sync_playwright

def firefox_binary():
 if os.environ.get('FIREFOX_BINARY'):return os.environ['FIREFOX_BINARY']
 with sync_playwright() as p:return p.firefox.executable_path

def geckodriver():
 return os.environ.get('GECKODRIVER') or shutil.which('geckodriver') or '/tmp/limitlessx-tools/geckodriver'
