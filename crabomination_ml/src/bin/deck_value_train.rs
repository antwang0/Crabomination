//! `deck_value_train` — step 1 of the deck-building net (ML_NOTES "A deck net
//! that builds decks — the plan, and step 1 pre-registered"): does a
//! context-aware deck value on the play net's card encoder predict sealed
//! matches better than the additive rating table?
//!
//! Reads a rating run's directory (`fra_card_eval` writes `decks.tsv` and
//! `games.tsv`; `fra_card_eval --dump-cards DIR` adds `card_feats.tsv` and
//! `deck_feats.tsv`) and fits one arm:
//!
//! | arm | deck value `f(deck)` |
//! |---|---|
//! | `A0`  | `Σ copies · a_card`, one scalar per card name — the rating table's model |
//! | `A0f` | A0 plus an MLP over the 16 deck features (diagnostic, not registered) |
//! | `A1`  | the play net's object layer on each card (frozen) → per-card value, context interaction, deck term |
//! | `A2`  | A1 with the object layer and the cards' embedding rows trained at lr × 0.1 |
//! | `A3`  | A1's shape from a random init, trained |
//! | `A4`  | A1 without the context: `Σ copies · a(x_card)` |
//!
//! The context model is `f = Σ_c n_c (a(x_c) + B(x_c) · c(ctx)) + h(ctx)`,
//! `ctx` the deck's mean card vector ⊕ its deck features — a rank-16
//! interaction between each card and the deck around it. Each game of a match
//! is won by A with `σ(f(A) − f(B))`. Decks are split 80 / 20 by a fixed hash:
//! train on matches between training decks, early-stop and pick λ on mixed
//! matches, report on matches between held-out decks. Full batch on the CPU.
//! One JSON line per run is appended to `--out`.
//!
//! ```text
//! deck_value_train --data .ladder/deckval/fra --net nets/fra-champion.safetensors \
//!     --arm A1 --lambda 10 --seed 1 --out .ladder/deckval/fra/runs.jsonl
//! ```

use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

use candle_core::{DType, Device, Result as CResult, Tensor, Var};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

const FEATS: usize = 59;
const DECK_FEATS: usize = 16;
const RANK: usize = 16;
const HID: usize = 32;

struct Args {
    data: PathBuf,
    net: Option<PathBuf>,
    arm: String,
    lambda: f64,
    seed: u64,
    lr: f64,
    steps: usize,
    out: PathBuf,
    /// Write the best-val model here (`crabomination::deck_value::DeckValue`).
    export: Option<PathBuf>,
}

fn parse_args() -> Args {
    let a: Vec<String> = std::env::args().collect();
    let get = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1).cloned());
    let arm = get("--arm").expect("--arm A0|A0f|A1|A2|A3|A4");
    let convex = arm.starts_with("A0");
    Args {
        data: PathBuf::from(get("--data").expect("--data DIR")),
        net: get("--net").map(PathBuf::from),
        lambda: get("--lambda").map_or(10.0, |v| v.parse().expect("--lambda")),
        seed: get("--seed").map_or(1, |v| v.parse().expect("--seed")),
        lr: get("--lr").map_or(if convex { 0.05 } else { 0.003 }, |v| v.parse().expect("--lr")),
        steps: get("--steps").map_or(4000, |v| v.parse().expect("--steps")),
        out: PathBuf::from(get("--out").expect("--out FILE.jsonl")),
        export: get("--export").map(PathBuf::from),
        arm,
    }
}

fn splitmix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// The fixed deck split, shared by every arm and seed: one deck in five held out.
fn held_out(deck: usize) -> bool {
    splitmix(deck as u64 ^ 0xDEC_C5A1_7000).is_multiple_of(5)
}

struct Data {
    names: Vec<String>,
    /// Per card: vocabulary index and its 59 hand-object features.
    cards: Vec<(u16, [f32; FEATS])>,
    /// `[decks, cards]` copy counts.
    counts: Vec<f32>,
    feats: Vec<f32>,
    decks: usize,
    /// `(deck a, deck b, games a won, games b won)`.
    matches: Vec<(u32, u32, f32, f32)>,
}

fn load(dir: &std::path::Path) -> Data {
    let read = |f: &str| std::fs::read_to_string(dir.join(f)).unwrap_or_else(|e| panic!("{}: {e}", dir.join(f).display()));
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut cards = Vec::new();
    let mut names = Vec::new();
    for line in read("card_feats.tsv").lines() {
        let col: Vec<&str> = line.split('\t').collect();
        assert_eq!(col.len(), 2 + FEATS, "card_feats.tsv: {line}");
        let mut f = [0f32; FEATS];
        for (v, c) in f.iter_mut().zip(&col[2..]) {
            *v = c.parse().expect("feature");
        }
        index.insert(col[0].to_string(), cards.len());
        names.push(col[0].to_string());
        cards.push((col[1].parse().expect("vocab index"), f));
    }
    let decks_text = read("decks.tsv");
    let decks = decks_text.lines().count();
    let mut counts = vec![0f32; decks * cards.len()];
    for line in decks_text.lines() {
        let mut col = line.split('\t');
        let d: usize = col.next().expect("id").parse().expect("deck id");
        for name in col {
            counts[d * cards.len() + index[name]] += 1.0;
        }
    }
    let mut feats = vec![0f32; decks * DECK_FEATS];
    for line in read("deck_feats.tsv").lines() {
        let col: Vec<&str> = line.split('\t').collect();
        let d: usize = col[0].parse().expect("deck id");
        for (k, c) in col[1..].iter().enumerate() {
            feats[d * DECK_FEATS + k] = c.parse().expect("deck feature");
        }
    }
    let matches = read("games.tsv")
        .lines()
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            (c[0].parse().unwrap(), c[1].parse().unwrap(), c[2].parse().unwrap(), c[3].parse().unwrap())
        })
        .collect();
    Data { names, cards, counts, feats, decks, matches }
}

/// One split's matches as tensors.
struct Split {
    a: Tensor,
    b: Tensor,
    wa: Tensor,
    wb: Tensor,
    games: f64,
    /// Decisive matches (`wa != wb`) and which side won them, for accuracy.
    sign: Vec<f32>,
}

fn split(m: &[(u32, u32, f32, f32)], keep: impl Fn(usize, usize) -> bool, dev: &Device) -> CResult<Split> {
    let m: Vec<_> = m.iter().copied().filter(|&(a, b, _, _)| keep(a as usize, b as usize)).collect();
    let games = m.iter().map(|x| (x.2 + x.3) as f64).sum();
    Ok(Split {
        a: Tensor::new(m.iter().map(|x| x.0).collect::<Vec<_>>(), dev)?,
        b: Tensor::new(m.iter().map(|x| x.1).collect::<Vec<_>>(), dev)?,
        wa: Tensor::new(m.iter().map(|x| x.2).collect::<Vec<_>>(), dev)?,
        wb: Tensor::new(m.iter().map(|x| x.3).collect::<Vec<_>>(), dev)?,
        games,
        sign: m.iter().map(|x| (x.2 - x.3).signum()).collect(),
    })
}

/// `log σ(x)`, stably.
fn log_sigmoid(x: &Tensor) -> CResult<Tensor> {
    // −softplus(−x) = min(x, 0) − log(1 + e^{−|x|})
    let neg_part = x.minimum(0f32)?;
    neg_part - (x.abs()?.neg()?.exp()? + 1.0)?.log()?
}

/// Negative log-likelihood per game of `f`'s predictions on `s`.
fn nll(f: &Tensor, s: &Split) -> CResult<Tensor> {
    let d = (f.index_select(&s.a, 0)? - f.index_select(&s.b, 0)?)?;
    let ll = ((s.wa.mul(&log_sigmoid(&d)?))? + s.wb.mul(&log_sigmoid(&d.neg()?)?)?)?;
    ll.sum_all()?.neg()? / s.games
}

fn accuracy(f: &Tensor, s: &Split) -> CResult<f64> {
    let d: Vec<f32> = (f.index_select(&s.a, 0)? - f.index_select(&s.b, 0)?)?.to_vec1()?;
    let (mut hit, mut n) = (0usize, 0usize);
    for (d, s) in d.iter().zip(&s.sign) {
        if *s != 0.0 {
            n += 1;
            hit += usize::from(d.signum() == *s);
        }
    }
    Ok(hit as f64 / n.max(1) as f64)
}

/// A dense layer over trainable vars.
struct Lin {
    w: Var,
    b: Var,
}

/// A `[r, c]` var drawn uniformly from `±s` off the run's own rng — candle's
/// CPU backend cannot be seeded, and an arm's seeds must replay.
fn uniform(rng: &mut StdRng, s: f32, (r, c): (usize, usize), dev: &Device) -> CResult<Var> {
    Var::from_tensor(&Tensor::from_vec((0..r * c).map(|_| rng.random_range(-s..s)).collect::<Vec<f32>>(), (r, c), dev)?)
}

impl Lin {
    fn new(i: usize, o: usize, rng: &mut StdRng, dev: &Device) -> CResult<Self> {
        let s = (1.0 / i as f32).sqrt();
        Ok(Self { w: uniform(rng, s, (o, i), dev)?, b: Var::zeros(o, DType::F32, dev)? })
    }
    fn fwd(&self, x: &Tensor) -> CResult<Tensor> {
        x.matmul(&self.w.as_tensor().t()?)?.broadcast_add(self.b.as_tensor())
    }
    fn vars(&self) -> Vec<Var> {
        vec![self.w.clone(), self.b.clone()]
    }
    fn export(&self) -> CResult<crabomination::deck_value::Layer> {
        Ok(crabomination::deck_value::Layer { w: self.w.as_tensor().to_vec2()?, b: self.b.as_tensor().to_vec1()? })
    }
}

/// The per-card value, the card's interaction vector, the context map and
/// the deck term — everything above the card encoder.
struct Head {
    a1: Lin,
    a2: Lin,
    bmap: Lin,
    c1: Lin,
    c2: Lin,
    h1: Lin,
    h2: Lin,
}

impl Head {
    fn new(h: usize, rng: &mut StdRng, dev: &Device) -> CResult<Self> {
        let ctx = h + DECK_FEATS;
        Ok(Self {
            a1: Lin::new(h, HID, rng, dev)?,
            a2: Lin::new(HID, 1, rng, dev)?,
            bmap: Lin::new(h, RANK, rng, dev)?,
            c1: Lin::new(ctx, HID, rng, dev)?,
            c2: Lin::new(HID, RANK, rng, dev)?,
            h1: Lin::new(ctx, HID, rng, dev)?,
            h2: Lin::new(HID, 1, rng, dev)?,
        })
    }
    fn vars(&self, context: bool) -> Vec<Var> {
        let mut v = [&self.a1, &self.a2].iter().flat_map(|l| l.vars()).collect::<Vec<_>>();
        if context {
            for l in [&self.bmap, &self.c1, &self.c2, &self.h1, &self.h2] {
                v.extend(l.vars());
            }
        }
        v
    }
    /// The per-card quantities and context layers, as the engine reads them.
    fn export(&self, x: &Tensor, arm: &str, names: &[String], context: bool) -> CResult<crabomination::deck_value::DeckValue> {
        let a: Vec<f32> = self.a2.fwd(&self.a1.fwd(x)?.relu()?)?.squeeze(1)?.to_vec1()?;
        let mut dv = crabomination::deck_value::DeckValue::new(arm, names.to_vec(), a);
        if context {
            dv.x = x.to_vec2()?;
            dv.b = self.bmap.fwd(x)?.to_vec2()?;
            dv.ctx = Some([self.c1.export()?, self.c2.export()?, self.h1.export()?, self.h2.export()?]);
        }
        Ok(dv)
    }

    /// `f` per deck from the card vectors `x` `[C, h]`.
    fn value(&self, x: &Tensor, counts: &Tensor, feats: &Tensor, context: bool) -> CResult<Tensor> {
        let a = self.a2.fwd(&self.a1.fwd(x)?.relu()?)?; // [C, 1]
        let mut f = counts.matmul(&a)?.squeeze(1)?; // [D]
        if context {
            let n = counts.sum_keepdim(1)?; // [D, 1]
            let mean = counts.matmul(x)?.broadcast_div(&n)?; // [D, h]
            let ctx = Tensor::cat(&[&mean, feats], 1)?;
            let c = self.c2.fwd(&self.c1.fwd(&ctx)?.relu()?)?; // [D, R]
            let mb = counts.matmul(&self.bmap.fwd(x)?)?; // [D, R]
            f = (f + mb.mul(&c)?.sum(1)?)?;
            f = (f + self.h2.fwd(&self.h1.fwd(&ctx)?.relu()?)?.squeeze(1)?)?;
        }
        Ok(f)
    }
}

/// The exported value read back by the engine must score the first decks as
/// the trainer did — the gate runs on the engine's reading, not this one.
fn parity(path: &std::path::Path, dir: &std::path::Path, want: &[f32]) {
    let dv = crabomination::deck_value::DeckValue::load(path).unwrap_or_else(|e| panic!("{e}"));
    let mut by_name: HashMap<&'static str, crabomination::cube::CardFactory> = HashMap::new();
    for f in crabomination::catalog::sets::all_factories::all_catalog_card_factories() {
        let d = f();
        by_name.entry(d.name).or_insert(f);
        by_name.entry(d.name.split(" // ").next().unwrap_or(d.name)).or_insert(f);
    }
    let text = std::fs::read_to_string(dir.join("decks.tsv")).expect("decks.tsv");
    let mut worst = 0f64;
    for (line, &w) in text.lines().zip(want) {
        let deck: Vec<crabomination::cube::CardFactory> = line.split('\t').skip(1).map(|n| by_name[n]).collect();
        worst = worst.max((dv.value(&deck) - w as f64).abs());
    }
    assert!(worst < 1e-3, "engine and trainer disagree on a deck's value by {worst}");
}

fn main() -> CResult<()> {
    let args = parse_args();
    let dev = Device::Cpu;
    let data = load(&args.data);
    let c = data.cards.len();
    let counts = Tensor::from_vec(data.counts.clone(), (data.decks, c), &dev)?;
    let feats = Tensor::from_vec(data.feats.clone(), (data.decks, DECK_FEATS), &dev)?;
    let train = split(&data.matches, |a, b| !held_out(a) && !held_out(b), &dev)?;
    let val = split(&data.matches, |a, b| held_out(a) != held_out(b), &dev)?;
    let test = split(&data.matches, |a, b| held_out(a) && held_out(b), &dev)?;
    eprintln!(
        "{} cards, {} decks; matches train {} / val {} / test {} ({} / {} / {} games)",
        c,
        data.decks,
        train.sign.len(),
        val.sign.len(),
        test.sign.len(),
        train.games,
        val.games,
        test.games
    );
    let mut rng = StdRng::seed_from_u64(args.seed ^ 0xDEC0_DE5E);

    let card_feats = Tensor::from_vec(data.cards.iter().flat_map(|x| x.1).collect::<Vec<f32>>(), (c, FEATS), &dev)?;
    // The play net's object layer and the decks' cards' embedding rows.
    let net = |name: &str| -> CResult<Tensor> {
        let path = args.net.as_ref().expect("--net PLAY_NET for arms A1..A4");
        let mut t = candle_core::safetensors::load(path, &dev)?;
        Ok(t.remove(name).unwrap_or_else(|| panic!("{}: no {name}", path.display())))
    };
    let arm = args.arm.as_str();
    let (context, head_dim) = match arm {
        "A0" | "A0f" => (false, 0),
        "A1" | "A2" | "A3" => (true, 64),
        "A4" => (false, 64),
        other => panic!("unknown arm {other}"),
    };
    // Encoder: A1/A4 frozen from the net, A2 from the net and trained, A3
    // fresh and trained.
    let (enc_emb, enc_lin): (Option<Var>, Option<Lin>) = match arm {
        "A1" | "A2" | "A4" => {
            let emb = net("emb.weight")?;
            let idx = Tensor::new(data.cards.iter().map(|x| x.0 as u32).collect::<Vec<_>>(), &dev)?;
            let rows = emb.index_select(&idx, 0)?;
            (
                Some(Var::from_tensor(&rows)?),
                Some(Lin { w: Var::from_tensor(&net("obj.weight")?)?, b: Var::from_tensor(&net("obj.bias")?)? }),
            )
        }
        "A3" => (Some(uniform(&mut rng, 0.17, (c, 32), &dev)?), Some(Lin::new(32 + FEATS, 64, &mut rng, &dev)?)),
        _ => (None, None),
    };
    let encode = || -> CResult<Option<Tensor>> {
        match (&enc_emb, &enc_lin) {
            (Some(e), Some(l)) => Ok(Some(l.fwd(&Tensor::cat(&[e.as_tensor(), &card_feats], 1)?)?.relu()?)),
            _ => Ok(None),
        }
    };
    let enc_trained = matches!(arm, "A2" | "A3");
    let head = (head_dim > 0).then(|| Head::new(head_dim, &mut rng, &dev)).transpose()?;
    let scalar = Var::zeros((c, 1), DType::F32, &dev)?;
    let deck_mlp = (arm == "A0f")
        .then(|| -> CResult<(Lin, Lin)> { Ok((Lin::new(DECK_FEATS, HID, &mut rng, &dev)?, Lin::new(HID, 1, &mut rng, &dev)?)) })
        .transpose()?;

    let value = |x: Option<&Tensor>| -> CResult<Tensor> {
        match (&head, x) {
            (Some(h), Some(x)) => h.value(x, &counts, &feats, context),
            _ => {
                let mut f = counts.matmul(scalar.as_tensor())?.squeeze(1)?;
                if let Some((l1, l2)) = &deck_mlp {
                    f = (f + l2.fwd(&l1.fwd(&feats)?.relu()?)?.squeeze(1)?)?;
                }
                Ok(f)
            }
        }
    };
    let mut head_vars: Vec<Var> = match &head {
        Some(h) => h.vars(context),
        None => vec![scalar.clone()],
    };
    if let Some((l1, l2)) = &deck_mlp {
        head_vars.extend(l1.vars());
        head_vars.extend(l2.vars());
    }
    let enc_vars: Vec<Var> = if enc_trained {
        let mut v = vec![enc_emb.clone().expect("encoder")];
        v.extend(enc_lin.as_ref().expect("encoder").vars());
        v
    } else {
        Vec::new()
    };
    let mut opt_head = AdamW::new(head_vars.clone(), ParamsAdamW { lr: args.lr, weight_decay: 0.0, ..Default::default() })?;
    let enc_lr = if arm == "A2" { args.lr * 0.1 } else { args.lr };
    let mut opt_enc = (!enc_vars.is_empty())
        .then(|| AdamW::new(enc_vars.clone(), ParamsAdamW { lr: enc_lr, weight_decay: 0.0, ..Default::default() }))
        .transpose()?;
    // L2 on every trained weight matrix and scalar, per training game; the
    // pretrained encoder's is pulled toward its init, not zero, so A2 stays
    // a fine-tune.
    let enc_init: Vec<Tensor> = enc_vars.iter().map(|v| v.as_tensor().copy()).collect::<CResult<_>>()?;
    let penalty = |vars: &[Var], init: Option<&[Tensor]>| -> CResult<Tensor> {
        let mut p = Tensor::zeros((), DType::F32, &dev)?;
        for (i, v) in vars.iter().enumerate() {
            let t = match init {
                Some(init) => (v.as_tensor() - &init[i])?,
                None => v.as_tensor().clone(),
            };
            p = (p + t.sqr()?.sum_all()?)?;
        }
        Ok(p)
    };

    let t0 = std::time::Instant::now();
    let (mut best_val, mut best_step, mut best_test, mut best_acc) = (f64::INFINITY, 0usize, 0.0, 0.0);
    let mut train_at_best = 0.0;
    let mut best_export: Option<(crabomination::deck_value::DeckValue, Vec<f32>)> = None;
    let export_now = |x: Option<&Tensor>| -> CResult<crabomination::deck_value::DeckValue> {
        match (&head, x) {
            (Some(h), Some(x)) => h.export(x, arm, &data.names, context),
            _ => {
                let mut dv = crabomination::deck_value::DeckValue::new(arm, data.names.clone(), scalar.as_tensor().squeeze(1)?.to_vec1()?);
                if let Some((l1, l2)) = &deck_mlp {
                    dv.deck_mlp = Some([l1.export()?, l2.export()?]);
                }
                Ok(dv)
            }
        }
    };
    for step in 0..=args.steps {
        let x = encode()?;
        let f = value(x.as_ref())?;
        if step % 25 == 0 {
            let v = nll(&f, &val)?.to_scalar::<f32>()? as f64;
            if v < best_val {
                best_val = v;
                best_step = step;
                best_test = nll(&f, &test)?.to_scalar::<f32>()? as f64;
                best_acc = accuracy(&f, &test)?;
                train_at_best = nll(&f, &train)?.to_scalar::<f32>()? as f64;
                if args.export.is_some() {
                    best_export = Some((export_now(x.as_ref())?, f.narrow(0, 0, 64)?.to_vec1()?));
                }
            } else if step >= best_step + 600 {
                break;
            }
        }
        if step == args.steps {
            break;
        }
        let reg = (penalty(&head_vars, None)? + penalty(&enc_vars, Some(&enc_init))?)?;
        let loss = (nll(&f, &train)? + (reg * (args.lambda / train.games))?)?;
        let grads = loss.backward()?;
        opt_head.step(&grads)?;
        if let Some(o) = opt_enc.as_mut() {
            o.step(&grads)?;
        }
    }
    let line = format!(
        "{{\"data\":\"{}\",\"arm\":\"{}\",\"lambda\":{},\"seed\":{},\"lr\":{},\"best_step\":{},\"train_nll\":{:.6},\"val_nll\":{:.6},\"test_nll\":{:.6},\"test_acc\":{:.4},\"secs\":{:.0}}}",
        args.data.display(),
        arm,
        args.lambda,
        args.seed,
        args.lr,
        best_step,
        train_at_best,
        best_val,
        best_test,
        best_acc,
        t0.elapsed().as_secs_f64()
    );
    println!("{line}");
    if let (Some(path), Some((dv, want))) = (&args.export, &best_export) {
        dv.save(path).unwrap_or_else(|e| panic!("{e}"));
        parity(path, &args.data, want);
        eprintln!("exported {} to {} (engine parity checked on {} decks)", dv.arm, path.display(), want.len());
    }
    let mut out = std::fs::OpenOptions::new().create(true).append(true).open(&args.out).expect("--out");
    writeln!(out, "{line}").expect("write --out");
    Ok(())
}
