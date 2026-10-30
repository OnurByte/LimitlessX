"""Recovery regressions: partial feeds, shared identity errors, replies and sessions."""
import copy
import json
import pathlib
import tempfile
from urllib.parse import parse_qs, urlparse
from playwright.sync_api import sync_playwright, expect
from fixtures import USER, POST, html

ROOT = pathlib.Path(__file__).resolve().parents[2]

with sync_playwright() as p, tempfile.TemporaryDirectory(prefix='limitlessx-review-') as profile:
    ext = ROOT / 'dist/chromium'
    with p.chromium.launch_persistent_context(profile, channel='chromium', headless=True,
            args=[f'--disable-extensions-except={ext}', f'--load-extension={ext}', '--no-sandbox']) as ctx:
        state = {'feed_fail': True, 'identity_fail': True, 'reply_fail': True, 'version': 'old snapshot'}
        calls = []

        def user(handle, uid='12'):
            return {**USER, 'screen_name': handle, 'id': uid, 'name': handle, 'description': 'Public biography'}

        def post(handle, pid='20', text='public post'):
            return {**copy.deepcopy(POST), 'id': pid, 'text': text, 'author': user(handle)}

        def provider(route):
            url = route.request.url
            calls.append(url)
            parsed = urlparse(url)
            path = parsed.path
            query = parse_qs(parsed.query)
            if path == '/2/typeahead':
                handle = query['q'][0]
                if handle == 'identity' and state['identity_fail']:
                    route.abort('failed')
                else:
                    route.fulfill(json={'code': 200, 'users': [user(handle)]})
                return
            if path.startswith('/2/conversation/'):
                later = 'cursor' in query
                if later and state['reply_fail']:
                    route.fulfill(status=503, json={'code': 503})
                else:
                    route.fulfill(json={'code': 200, 'status': post('jack'), 'thread': [],
                        'replies': [post('jack', '22' if later else '21', 'second reply' if later else 'first reply')],
                        'cursor': {'bottom': None if later else 'next-replies'}})
                return
            handle = path.split('/')[3]
            feed = path.endswith('/statuses')
            if handle in ('identity', 'mismatch'):
                route.fulfill(status=404, json={'code': 404})
            elif handle.startswith('id:'):
                # Both recovered profiles resolve UID 12. Mismatch is tested after identity.
                if state.get('mismatch'):
                    route.fulfill(status=503, json={'code': 503}) if feed else route.fulfill(json={'code': 200, 'user': user('mismatch', '999')})
                else:
                    route.fulfill(json={'code': 200, **({'results': [post('identity')], 'cursor': {'bottom': None}} if feed else {'user': user('identity')})})
            elif feed and handle == 'partial' and state['feed_fail']:
                route.fulfill(status=503, json={'code': 503})
            elif feed:
                route.fulfill(json={'code': 200, 'results': [post(handle, text=state['version'] if handle == 'sessions' else 'public post')], 'cursor': {'bottom': None}})
            else:
                route.fulfill(json={'code': 200, 'user': user(handle)})

        ctx.route('https://api.fxtwitter.com/**', provider)
        def x_document(route):
            handle = urlparse(route.request.url).path.split('/')[1]
            notice = 'This post has been withheld in Türkiye in response to a legal demand.' if '/status/' in route.request.url else f'Account Withheld@{handle} has been withheld in TR in response to a legal demand.'
            route.fulfill(body=html(notice), content_type='text/html')
        ctx.route('https://x.com/**', x_document)
        worker = ctx.service_workers[0] if ctx.service_workers else ctx.wait_for_event('serviceworker')
        origin = '/'.join(worker.url.split('/')[:3])
        ui = ctx.new_page()
        ui.goto(origin + '/popup.html')
        ui.get_by_role('heading', name='LimitlessX', exact=True).wait_for()
        send = lambda message: ui.evaluate('(m)=>chrome.runtime.sendMessage(m)', message)
        x = ctx.new_page()

        x.goto('https://x.com/partial')
        retry = x.get_by_role('button', name='Try again', exact=True)
        retry.wait_for()
        state['feed_fail'] = False
        retry.click()
        expect(x.locator('#limitlessx-inline .post-text')).to_have_text('public post')

        x.goto('https://x.com/identity')
        retry.wait_for()
        failed_calls = sum('/typeahead' in url for url in calls)
        assert failed_calls >= 1
        state['identity_fail'] = False
        retry.click()
        expect(x.locator('#limitlessx-inline .post-text')).to_have_text('public post')
        assert sum('/typeahead' in url for url in calls) > failed_calls, 'failed shared identity result poisoned retry'

        x.goto('https://x.com/jack/status/20')
        expect(x.locator('#limitlessx-inline .post-text')).to_have_text(['public post', 'first reply'])
        x.get_by_role('button', name='Show more replies', exact=True).click()
        retry.wait_for()
        expect(x.locator('#limitlessx-inline .post-text')).to_have_text(['public post', 'first reply'])
        state['reply_fail'] = False
        retry.click()
        expect(x.locator('#limitlessx-inline .post-text')).to_have_text(['public post', 'first reply', 'second reply'])
        expect(x.locator('#limitlessx-inline .notice')).to_have_count(0)

        x.goto('https://x.com/sessions')
        expect(x.locator('#limitlessx-inline .post-text')).to_have_text('old snapshot')
        state['version'] = 'latest snapshot'
        x.bring_to_front()
        assert send({'op': 'open_current'})['ok']
        expect(x.locator('#limitlessx-inline .post-text')).to_have_text('latest snapshot')
        before_reload = len(calls)
        for _ in range(3):
            x.reload()
            expect(x.locator('#limitlessx-inline .post-text')).to_have_text('latest snapshot')
        assert len(calls) == before_reload, 'reload should restore latest session without refetching'

        state['mismatch'] = True
        x.goto('https://x.com/mismatch')
        x.get_by_role('alert').wait_for()
        expect(x.locator('#limitlessx-inline .x-profile')).to_have_count(0)
        expect(x.locator('#limitlessx-inline .post-card')).to_have_count(0)
        print(json.dumps({name: 'passed' for name in ['initial_timeline_retry', 'failed_identity_eviction', 'reply_retry_preserves_content', 'latest_session_restoration', 'recovered_uid_mismatch_rejected']}))
