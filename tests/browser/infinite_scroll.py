"""Five visible posts, parallel initial fetch, buffered scrolling and stable DOM."""
import copy,json,pathlib,tempfile
from playwright.sync_api import sync_playwright
from fixtures import POST,USER,html
ROOT=pathlib.Path(__file__).resolve().parents[2]
with sync_playwright() as p,tempfile.TemporaryDirectory(prefix='limitlessx-scroll-') as profile:
 ext=ROOT/'dist/chromium'
 with p.chromium.launch_persistent_context(profile,channel='chromium',headless=True,args=[f'--disable-extensions-except={ext}',f'--load-extension={ext}','--no-sandbox']) as ctx:
  calls=[];pending=[];failed=[];short=[False]
  def provider(route):
   url=route.request.url;calls.append(url)
   if '/statuses' not in url:
    pending.append(route);return
   if 'cursor=' in url and not failed:
    failed.append(True);route.fulfill(status=503,json={'code':503});return
   start,end=(21,26) if 'cursor=' in url else (1,21)
   posts=[{**copy.deepcopy(POST),'id':str(i),'text':f'Post {i}\n'+('Public fixture text\n'*(0 if short[0] else 14)),'replying_to':{'screen_name':'someone'} if i%2==0 else None,'author':{**USER,'screen_name':'other' if i%3==0 else 'jack'},'media':{'photos':[{'type':'photo','url':'https://pbs.twimg.com/media/fixture.jpg'}]} if i%4==0 else {}}for i in range(start,end)]
   route.fulfill(json={'code':200,'results':posts,'cursor':{'bottom':None if 'cursor=' in url else 'next'}})
   for r in pending:r.fulfill(json={'code':200,'user':USER})
   pending.clear()
  ctx.route('https://api.fxtwitter.com/**',provider)
  ctx.route('https://pbs.twimg.com/**',lambda r:r.abort())
  ctx.route('https://x.com/**',lambda r:r.fulfill(body=html('Account Withheld@jack has been withheld in TR in response to a legal demand.'),content_type='text/html'))
  x=ctx.new_page();x.set_viewport_size({'width':1100,'height':600});x.goto('https://x.com/jack')
  x.locator('#limitlessx-inline .x-profile').wait_for();cards=x.locator('#limitlessx-inline .post-card');assert cards.count()==5
  assert len(calls)==2,'initial profile and timeline were not fetched together'
  x.wait_for_timeout(700);assert cards.count()==5,'initial screen eagerly loaded more than five'
  x.evaluate('globalThis.firstPost=document.querySelector("#limitlessx-inline").shadowRoot.querySelector(".post-card")')
  for count in [10,15,20]:
   x.mouse.wheel(0,100000)
   x.wait_for_function('(count)=>document.querySelector("#limitlessx-inline").shadowRoot.querySelectorAll(".post-card").length===count',arg=count)
   assert x.evaluate('globalThis.firstPost===document.querySelector("#limitlessx-inline").shadowRoot.querySelector(".post-card")'),'existing post DOM was rebuilt'
   if count<=20:assert sum('/statuses'in u for u in calls)==1,'buffered posts caused a provider request'
  x.mouse.wheel(0,100000);x.get_by_role('button',name='Try again').wait_for();assert cards.count()==20,'pagination failure destroyed existing posts'
  x.get_by_role('button',name='Try again').click();x.wait_for_function('document.querySelector("#limitlessx-inline").shadowRoot.querySelectorAll(".post-card").length===25')
  assert sum('/statuses'in u for u in calls)==3
  assert x.locator('[data-autoload]').count()==0
  assert len(set(cards.evaluate_all('(nodes)=>nodes.map(n=>n.dataset.postId)')))==25
  before_tabs=len(calls)
  for label,path,ids in [('Replies','with_replies',['2','4','6','8','10']),('Media','media',['4','8','12','16','20']),('Reposts','reposts',['3','6','9','12','15'])]:
   x.get_by_role('link',name=label,exact=True).click();x.wait_for_url('**/'+path)
   x.wait_for_function('(label)=>document.querySelector("#limitlessx-inline")?.shadowRoot?.querySelector(".x-tab.active")?.textContent===label',arg=label)
   assert cards.evaluate_all('(nodes)=>nodes.map(n=>n.dataset.postId)')==ids
  assert len(calls)==before_tabs,'profile tab switch refetched an existing buffer'
  short[0]=True
  z=ctx.new_page();z.set_viewport_size({'width':1100,'height':2500});z.goto('https://x.com/jack');z.locator('#limitlessx-inline .x-profile').wait_for()
  assert z.locator('#limitlessx-inline .post-card').count()==5
  assert z.evaluate('document.documentElement.scrollHeight<=innerHeight'),'short-feed fixture unexpectedly scrolls'
  z.mouse.wheel(0,100)
  z.wait_for_function('document.querySelector("#limitlessx-inline").shadowRoot.querySelectorAll(".post-card").length===10')
  print(json.dumps({'initial_five':'passed','parallel_initial_requests':'passed','scroll_batches':'passed','buffer_without_refetch':'passed','stable_existing_posts':'passed','cursor_end':'passed','pagination_error_preserves_feed':'passed','short_feed_wheel':'passed','same_tab_profile_filters':'passed'}))
