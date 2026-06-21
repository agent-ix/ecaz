def variant:
  if .step | test("source-f32") then "source/f32"
  elif .step | test("index-f16") then "index/f16"
  elif .step | test("index-rabitq4") then "index/rabitq4"
  elif .step | test("index-rabitq8") then "index/rabitq8"
  elif .step | test("index-turboquant") then "index/turboquant"
  else "unknown" end;

def width:
  (.step | capture("w(?<w>[0-9]+)").w | tonumber);

def msnum($s):
  ($s | sub(" ms$"; "") | tonumber);

def rkey($r):
  "\($r.variant)|\($r.width)|\($r.nprobe)";

def skey($r):
  "\($r.variant)|\($r.width)";

def selected($rows; $target; $variant):
  ($rows
    | map(select(.variant == $variant and .recall >= $target))
    | sort_by(.p50)
    | first) as $hit
  | if $hit then
      $hit + {target: $target, status: "hit"}
    else
      ($rows
        | map(select(.variant == $variant))
        | sort_by(-.recall, .p50)
        | first) + {target: $target, status: "NO_REACH"}
    end;

([.[] | select(.metric == "latency") | {
  variant: variant,
  width: width,
  nprobe: (.values.nprobe | tonumber),
  p50: msnum(.values.p50),
  p95: msnum(.values.p95),
  p99: msnum(.values.p99),
  mean: msnum(.values.mean)
}] | INDEX(rkey(.))) as $lat
| ([.[] | select(.metric == "storage_index" and .values["access method"] == "ec_ivf") | {
  variant: variant,
  width: width,
  index_size: .values.size,
  index_per_row: .values.per_row_bytes
}] | INDEX(skey(.))) as $stor
| ([.[] | select(.metric == "recall") | {
  variant: variant,
  width: width,
  nprobe: (.values.nprobe | tonumber),
  recall: (.values["recall@k"] | tonumber),
  ndcg: (.values["ndcg@k"] | tonumber),
  recall_qtime: .values["mean q-time"]
}] | INDEX(rkey(.))) as $rec
| ($rec | to_entries | map(.value | . + ($lat[rkey(.)] // {}) + ($stor[skey(.)] // {}))) as $rows
| (["source/f32", "index/f16", "index/rabitq4", "index/rabitq8", "index/turboquant"] as $variants
   | [0.95, 0.97, 0.99] as $targets
   | [$variants[] as $v | $targets[] as $t | selected($rows; $t; $v)]) as $sel
| ($sel | sort_by(.target, .variant)) as $out
| ("target\tvariant\tstatus\trow\trecall\tp50_ms\tp95_ms\tp99_ms\tindex_size"),
  ($out[] | [
    .target,
    .variant,
    .status,
    ("w\(.width) np\(.nprobe)"),
    (.recall | tostring),
    (.p50 | tostring),
    (.p95 | tostring),
    (.p99 | tostring),
    .index_size
  ] | @tsv)
