"""Packaged controls use public X media URLs and retain native media state."""
import copy,json,pathlib,tempfile
from playwright.sync_api import sync_playwright
from fixtures import POST,USER,html
ROOT=pathlib.Path(__file__).resolve().parents[2]
with sync_playwright() as p,tempfile.TemporaryDirectory(prefix='limitlessx-player-') as profile:
 ext=ROOT/'dist/chromium'
 with p.chromium.launch_persistent_context(profile,channel='chromium',headless=True,args=[f'--disable-extensions-except={ext}',f'--load-extension={ext}','--no-sandbox']) as ctx:
  failed=[False]
  media={'videos':[{'type':'video','url':'https://video.twimg.com/fixture.webm'}]}
  posts=[{**copy.deepcopy(POST),'id':str(i),'text':f'Post {i}\n'+('Public fixture\n'*12),'media':media if i==20 else {}}for i in range(20,30)]
  def provider(r):
   u=r.request.url
   if '/statuses'in u:r.fulfill(json={'code':200,'results':posts,'cursor':{'bottom':None}})
   else:r.fulfill(json={'code':200,'user':USER})
  def video(r):
   if failed[0]:r.fulfill(status=404,headers={'Access-Control-Allow-Origin':'*'})
   else:
    data=(ROOT/'tests/fixtures/player.webm').read_bytes();header=r.request.headers.get('range');start=0;end=len(data)-1
    if header:
     bounds=header.removeprefix('bytes=').split('-');start=int(bounds[0] or 0);end=min(int(bounds[1]) if bounds[1] else end,end)
    headers={'Access-Control-Allow-Origin':'*','Accept-Ranges':'bytes','Content-Length':str(end-start+1)}
    if header:headers['Content-Range']=f'bytes {start}-{end}/{len(data)}'
    r.fulfill(status=206 if header else 200,body=data[start:end+1],content_type='video/webm',headers=headers)
  ctx.route('https://api.fxtwitter.com/**',provider);ctx.route('https://video.twimg.com/**',video)
  ctx.route('https://x.com/**',lambda r:r.fulfill(body=html('Account Withheld@jack has been withheld in TR in response to a legal demand.'),content_type='text/html'))
  x=ctx.new_page();x.set_viewport_size({'width':1000,'height':700});x.goto('https://x.com/jack')
  play=x.get_by_role('button',name='Play video',exact=True);play.wait_for();v=x.locator('video');assert not v.evaluate('(v)=>v.controls')
  assert x.get_by_label('Playback speed').input_value()=='1'
  play.click();x.wait_for_function('!document.querySelector("#limitlessx-inline").shadowRoot.querySelector("video").paused')
  x.get_by_role('button',name='Pause video',exact=True).click();assert v.evaluate('(v)=>v.paused')
  x.get_by_label('Playback speed').select_option('1.5');assert v.evaluate('(v)=>v.playbackRate')==1.5
  x.get_by_role('button',name='Mute video',exact=True).click();assert v.evaluate('(v)=>v.muted')
  x.wait_for_function('Number.isFinite(document.querySelector("#limitlessx-inline").shadowRoot.querySelector("video").duration)')
  x.get_by_label('Seek video').evaluate('(n)=>{n.value="500";n.dispatchEvent(new Event("input",{bubbles:true}))}');assert 5<=v.evaluate('(v)=>v.currentTime')<=7, v.evaluate('(v)=>({time:v.currentTime,duration:v.duration,ready:v.readyState})')
  x.get_by_role('button',name='Full screen',exact=True).click();assert x.evaluate('!!document.fullscreenElement')
  x.get_by_role('button',name='Exit full screen',exact=True).click();x.wait_for_function('!document.fullscreenElement')
  x.evaluate('globalThis.savedVideo=document.querySelector("#limitlessx-inline").shadowRoot.querySelector("video")')
  x.get_by_role('button',name='Load more posts').click();x.wait_for_function('document.querySelector("#limitlessx-inline").shadowRoot.querySelectorAll(".post-card").length===10')
  assert x.evaluate('globalThis.savedVideo===document.querySelector("#limitlessx-inline").shadowRoot.querySelector("video")')
  assert v.evaluate('(v)=>v.playbackRate')==1.5
  # Media failure keeps a source link and can be retried without replacing the reader.
  failed[0]=True;v.evaluate('(v)=>{v.src="https://video.twimg.com/fixture.webm?fail=1";v.load()}')
  x.get_by_role('button',name='Retry video').wait_for();assert x.get_by_role('link',name='Open video on X').is_visible()
  failed[0]=False;x.get_by_role('button',name='Retry video').click();x.wait_for_function('!document.querySelector("#limitlessx-inline").shadowRoot.querySelector("video").paused')
  x.evaluate("history.pushState({},'', '/other');document.body.append(document.createElement('div'))");x.locator('#limitlessx-inline').wait_for(state='detached');assert x.evaluate('globalThis.savedVideo.paused')
  print(json.dumps({'play_pause':'passed','seek_volume_speed':'passed','fullscreen':'passed','state_preserved_on_append':'passed','error_retry':'passed','navigation_pauses':'passed'}))
