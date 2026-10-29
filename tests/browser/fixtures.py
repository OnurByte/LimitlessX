USER={'type':'profile','id':'12','screen_name':'jack','name':'jack','description':'<script>globalThis.injected=true</script>','protected':False,'avatar_url':None}
POST={'type':'status','provider':'twitter','id':'20','text':'just setting up my twttr','author':USER,'created_at':None,'media':{}}
NOTICE="This post has been withheld in Türkiye in response to a legal demand."
def html(text=NOTICE):
 return f'<html><head><meta charset="utf-8"></head><body><main><div data-testid="primaryColumn"><div data-testid="emptyState"><span>{text}</span><a href="https://help.x.com/en/rules-and-policies/post-withheld-by-country">Learn more</a></div></div></main></body></html>'
