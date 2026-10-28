#!/usr/bin/env python3
"""Print `card_ratings::FRA_SIM_VALUE` from a `fra_card_eval.py` ratings.tsv.

usage: fra_card_table.py RATINGS_TSV MIN_DECKS [--tsv]

--tsv prints `name<TAB>score` in builder units (OFFSET + SCALE * value,
rounded) for every rated card, `-` for one under MIN_DECKS — the override
`fra_card_eval --ab --table FILE` races against the compiled table.

The first line is a comment with SCALE / OFFSET: the listed cards' static
scores' mean and spread matched to their values'. Lands are left out (the
builder never reads a land's rating).
"""
import csv, math, statistics as st, sys
LANDS={'Deserted Beach','Shipwreck Marsh','Haunted Ridge','Rockfall Vale','Overgrown Farmland','Dedicated Commons','Formidable Commons','Meticulous Commons','Innovative Commons','Transformative Commons','Fatehold Annex','Konstrari Annex','Stingerquill Annex','Theorix Annex','Vigorbloom Annex','Room of Refuge','Hall of Echoes','Hexhaven Dueling Arena','Roiling Canopy',"Theorist's Sanctum"}
allrows=[r for r in csv.DictReader(open(sys.argv[1]),delimiter='\t') if r['name'] not in LANDS]
rows=[r for r in allrows if int(r['decks'])>=int(sys.argv[2])]
vals=[float(r['value']) for r in rows]; stat=[float(r['static']) for r in rows]
scale=st.pstdev(stat)/st.pstdev(vals); off=st.mean(stat)-scale*st.mean(vals)
if '--tsv' in sys.argv:
    for r in allrows:
        ok=int(r['decks'])>=int(sys.argv[2])
        print(f"{r['name']}\t{math.floor(off+scale*float(r['value'])+0.5) if ok else '-'}")
    sys.exit(0)
print(f"// n={len(rows)} static {st.mean(stat):.2f} ± {st.pstdev(stat):.2f}, value {st.mean(vals):.2f} ± {st.pstdev(vals):.2f}; SCALE {scale:.3f} OFFSET {off:.3f}")
for r in sorted(rows,key=lambda r:r['name']):
    n=r['name'].replace('\\','\\\\').replace('"','\\"')
    print(f'    ("{n}", {float(r["value"]):.2f}),')
