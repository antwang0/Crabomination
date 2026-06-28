import json, urllib.request, urllib.parse, time, sys
from pathlib import Path
names=[l.strip() for l in open("scratch_names.txt") if l.strip()]
out=[]
# batched via collection endpoint
ids=[{"name":n} for n in names]
for i in range(0,len(ids),75):
    chunk=ids[i:i+75]
    body=json.dumps({"identifiers":chunk}).encode()
    req=urllib.request.Request("https://api.scryfall.com/cards/collection",data=body,
        headers={"Content-Type":"application/json","User-Agent":"crab/1.0","Accept":"application/json"})
    data=json.load(urllib.request.urlopen(req,timeout=30))
    out.extend(data["data"])
    time.sleep(0.1)
res=[]
for c in out:
    res.append({"name":c["name"],"cost":c.get("mana_cost",""),"type":c.get("type_line",""),
        "pt":(f'{c.get("power")}/{c.get("toughness")}' if c.get("power") is not None else ""),
        "loy":c.get("loyalty",""),
        "text":(c.get("oracle_text","") or "").replace("\n"," | ")})
json.dump(res,open("scratch_mh3.json","w"),indent=0)
print("fetched",len(res))
