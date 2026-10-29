#!/usr/bin/env python3
"""Explicit, credential-free live feasibility probes. Never part of application startup."""
import argparse, datetime, json, urllib.request, urllib.error
from pathlib import Path
CASES = {
 'normal_post': '/2/status/20',
 'normal_profile': '/2/profile/github',
 'historically_withheld_profile': '/2/profile/ekrem_imamoglu',
 'historically_withheld_timeline': '/2/profile/ekrem_imamoglu/statuses',
 'protected_candidate': '/2/profile/asmallteapot',
 'suspended_candidate': '/2/profile/0',
 'missing_candidate': '/2/profile/lxnonexistent9z',
 'deleted_or_missing_post': '/2/status/1234567890123456789',
 'invalid_request': '/2/status/invalid',
}
def run():
 parser=argparse.ArgumentParser(); parser.add_argument('--output',default='docs/verification/feasibility.json');args=parser.parse_args()
 report={'checked_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'provider':'FxTwitter v2','restriction_provenance':'https://www.hrw.org/news/2025/05/08/social-media-companies-should-resist-turkish-state-censorship','limitations':['Historical withholding is documented; current X DOM restriction must be separately verified.','Candidates are not assumed private, deleted or suspended unless the response explicitly identifies that state.','One successful public lookup is not a guarantee of access.'],'cases':[]}
 for name,path in CASES.items():
  req=urllib.request.Request('https://api.fxtwitter.com'+path,headers={'User-Agent':'LimitlessX-feasibility/0.1','Accept':'application/json'})
  try:
   try:r=urllib.request.urlopen(req,timeout=20)
   except urllib.error.HTTPError as e:r=e
   with r:
    raw=r.read(4*1024*1024);data=json.loads(raw)
    row={'case':name,'path':path,'http_status':r.status,'code':data.get('code'),'message':data.get('message'),'reason':data.get('reason'),'has_user':isinstance(data.get('user'),dict),'protected':(data.get('user') or {}).get('protected'),'has_post':(data.get('status') or {}).get('type')=='status','post_count':len(data.get('results',[])),'has_next_cursor':bool((data.get('cursor') or {}).get('bottom'))}
  except Exception as e:row={'case':name,'path':path,'error':str(e)}
  report['cases'].append(row);print(name,row.get('http_status',row.get('error')),flush=True)
 Path(args.output).parent.mkdir(parents=True,exist_ok=True);Path(args.output).write_text(json.dumps(report,indent=2)+'\n')
if __name__=='__main__':run()
