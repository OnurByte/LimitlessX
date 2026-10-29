"""Literal geometry expectations are taken from the actual Helium X capture."""
import copy,json,pathlib,tempfile
from playwright.sync_api import sync_playwright
from fixtures import USER,POST,html
ROOT=pathlib.Path(__file__).resolve().parents[2]
with sync_playwright() as p,tempfile.TemporaryDirectory(prefix='limitlessx-fidelity-') as profile:
 ext=ROOT/'dist/chromium'
 with p.chromium.launch_persistent_context(profile,channel='chromium',headless=True,args=[f'--disable-extensions-except={ext}',f'--load-extension={ext}','--no-sandbox']) as ctx:
  user={**USER,'followers':9876543,'following':820,'joined':'Sun Nov 27 19:45:16 +0000 2011','location':'İstanbul','banner_url':'https://pbs.twimg.com/profile_banners/12/1','avatar_url':'https://pbs.twimg.com/profile_images/12/a.jpg'}
  post={**copy.deepcopy(POST),'author':user,'likes':249,'replies':34,'reposts':10,'views':4137,'bookmarks':7}
  reply={**copy.deepcopy(post),'id':'21','text':'Reply from the source'};second={**copy.deepcopy(reply),'id':'22','text':'Second reply page'}
  calls=[]
  def provider(route):
   u=route.request.url;calls.append(u)
   if '/conversation/' in u:
    if '/21' in u:body={'code':200,'status':reply,'thread':None,'replies':[],'cursor':{'bottom':None}}
    else:body={'code':200,'status':post,'thread':None,'replies':[second]if 'cursor='in u else[reply],'cursor':{'bottom':None if 'cursor='in u else'next+/='}}
   elif '/statuses' in u:body={'code':200,'results':[post],'cursor':{'bottom':None}}
   elif '/profile/' in u:body={'code':200,'user':user}
   else:body={'code':200,'status':post}
   route.fulfill(json=body)
  ctx.route('https://api.fxtwitter.com/**',provider)
  ctx.route('https://pbs.twimg.com/**',lambda r:r.abort())
  ctx.route('https://x.com/**',lambda r:r.fulfill(body=html('Account Withheld@jack has been withheld in TR in response to a legal demand.' if r.request.url.endswith('/jack') else 'This post has been withheld in TR in response to a legal demand.'),content_type='text/html'))
  worker=ctx.service_workers[0] if ctx.service_workers else ctx.wait_for_event('serviceworker');origin='/'.join(worker.url.split('/')[:3]);ui=ctx.new_page();ui.goto(origin+'/popup.html');ui.get_by_role('heading',name='LimitlessX',exact=True).wait_for()
  x=ctx.new_page();x.set_viewport_size({'width':1000,'height':900});x.goto('https://x.com/jack/status/20');x.locator('[data-testid="primaryColumn"]').evaluate("n=>{n.style.width='600px';n.style.margin='0 auto'}")
  x.locator('#limitlessx-inline .post-card').nth(1).wait_for();detail=x.locator('#limitlessx-inline .detailed-post');comment=x.locator('#limitlessx-inline .post-card').nth(1)
  styles=lambda n:n.evaluate('(n)=>{let c=getComputedStyle(n);return {fontSize:c.fontSize,lineHeight:c.lineHeight,width:n.getBoundingClientRect().width,x:n.getBoundingClientRect().x}}')
  x.evaluate("history.replaceState({},'', '/i/status/20');dispatchEvent(new PopStateEvent('popstate'))");x.wait_for_timeout(600);x.locator('#limitlessx-inline .detailed-post').wait_for();
  x.locator('#limitlessx-inline').evaluate('n=>n.remove()');x.locator('#limitlessx-inline .detailed-post').wait_for();
  assert styles(detail.locator('.post-text'))['fontSize']=='17px';assert styles(detail.locator('.post-text'))['lineHeight']=='24px'
  assert styles(comment.locator('.post-text'))['fontSize']=='15px';assert styles(comment.locator('.post-text'))['lineHeight']=='20px'
  avatar=styles(comment.locator('.post-avatar'));text=styles(comment.locator('.post-text'));assert avatar['width']==40;assert text['x']-avatar['x']==48
  assert styles(detail.locator('.post-text'))['width']==568,'detail body is incorrectly kept in the avatar column'
  assert detail.locator('.x-metric-value').first.inner_text()=='34'
  x.get_by_role('button',name='Show more replies').click();x.get_by_text('Second reply page',exact=True).wait_for();assert x.locator('#limitlessx-inline .post-card').count()==3
  assert sum('/conversation/'in u for u in calls)==2
  comment.locator('.post-text').click();x.locator('#limitlessx-inline .detailed-post').get_by_text('Reply from the source',exact=True).wait_for()
  assert ui.evaluate('chrome.runtime.sendMessage({op:"record"})')['data']['posts']==1
  z=ctx.new_page();z.goto('https://x.com/jack');z.locator('#limitlessx-inline .x-profile').wait_for();assert z.locator('.x-profile-count').all_text_contents()==['820 Following','9.9M Followers'];assert z.locator('#limitlessx-inline .x-banner').count()==1
  art=ROOT/'tests/browser/artifacts';art.mkdir(exist_ok=True);z.screenshot(path=str(art/'native-profile.png'),full_page=True);x.screenshot(path=str(art/'native-comments.png'),full_page=True)
  print(json.dumps({'profile_metadata':'passed','main_post_17_24':'passed','replies_15_20':'passed','avatar_40_gap_8':'passed','full_width_detailed_body':'passed','reply_pagination':'passed','deliberate_reply_count':'passed','real_heading_detection':'passed'}))
