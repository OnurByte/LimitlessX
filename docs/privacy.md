# Privacy

LimitlessX has no managed backend, account registration, analytics or telemetry. It does not access X authentication cookies, passwords, private messages or broad browsing history.

When you deliberately open a profile or post, the extension sends its public identifier to api.fxtwitter.com. Opening an account also requests its public timeline. Explicit pagination requests transmit the provider's cursor. FxTwitter can see the request identifier and network information such as your IP address. Images and videos connect to public X media hosts. Their policies and availability are outside LimitlessX's control.

Extension Fetch requests omit credentials, use no referrer, and reject redirects. Media elements use anonymous cross-origin loading. No remote executable code is used.

The optional Record is stored locally in IndexedDB. It contains a random salt, recording preference, start date and unique salted identifier hashes for deliberately opened eligible profiles/posts. Counts derive from these stores. It stores no post bodies, handles or plaintext browsing history. Clear record deletes the ledger and rotates its salt. Disabling recording keeps existing counts while preventing future additions. Uninstalling deletes the browser-managed extension data.

Short-lived reader contexts are held in browser session storage, limited to 100 active sessions with a ten-minute lifetime. They include the requested target, restriction eligibility and IDs returned by the current timeline. They survive worker restarts and are cleared at browser shutdown. They are not part of The Record.

Permissions: `activeTab` reads the currently activated tab context; `storage` holds short-lived reader contexts in browser memory. Host access is limited to api.fxtwitter.com for explicit lookups. Content scripts run only on X/Twitter domains to observe restriction notices. No cookies, webRequest, history or all-sites permission is requested.

LimitlessX is not anonymous, untraceable, a VPN, a proxy service or a private-account viewer. Alternative public content can be incomplete, stale or unavailable.
