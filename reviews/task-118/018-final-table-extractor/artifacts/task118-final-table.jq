def scale:
  .step | capture("-(?<scale>[0-9]+k)-").scale;

def scale_rank:
  .scale | capture("(?<n>[0-9]+)k").n | tonumber;

def build_path:
  if (.step | contains("compressed-build")) then
    "compressed-build"
  else
    "source-build"
  end;

def storage_format:
  .values.storage_format // .values.quant // "";

def row_key:
  [scale, storage_format, build_path] | join("\u001f");

def format_rank:
  {
    "turboquant": 0,
    "pq_fastscan": 1,
    "rabitq": 2
  }[.format] // 99;

def build_rank:
  if .build_path == "source-build" then 0 else 1 end;

def init_row($r):
  .scale = ($r | scale)
  | .format = ($r | storage_format)
  | .build_path = ($r | build_path);

def add_row($r):
  if ($r.kind == "recall" and $r.values.ef_search == "200") then
    .recall_at_10 = $r.values["recall@k"]
    | .mean_q_time = $r.values["mean q-time"]
  elif $r.kind == "hnsw-frontier" then
    .truth10_in_frontier = $r.values["truth@10 in frontier"]
    | .truth100_in_frontier = $r.values["truth@100 in frontier"]
    | .exact_rerank = $r.values["exact rerank"]
    | .dropped_before_exact = $r.values["dropped before exact"]
  elif $r.kind == "hnsw-score-correlation" then
    .mean_spearman = $r.values["mean spearman"]
    | .mean_rank_shift = $r.values["mean |rank shift|"]
  elif ($r.kind == "storage" and $r.metric == "storage_field" and $r.values.field == "total") then
    .total_storage = $r.values.value
    | .total_storage_bytes = $r.values.value_bytes
  else
    .
  end;

def complete_enough_for_table:
  .scale and .format and .build_path;

([
  "Scale",
  "Format",
  "Build path",
  "Recall@10",
  "Mean q-time",
  "Truth@10 in frontier",
  "Truth@100 in frontier",
  "Exact rerank",
  "Dropped before exact",
  "Mean Spearman",
  "Mean rank shift",
  "Total storage",
  "Total storage bytes",
  "Dominant loss stage",
  "Next action"
] | @tsv),
(
  reduce .[] as $r ({};
    if (($r.step? // "") | test("-[0-9]+k-")) and (($r | storage_format) != "") then
      ($r | row_key) as $key
      | .[$key] = ((.[$key] // {}) | init_row($r) | add_row($r))
    else
      .
    end
  )
  | [ .[] | select(complete_enough_for_table) ]
  | sort_by(scale_rank, format_rank, build_rank)
  | .[]
  | [
      .scale,
      .format,
      .build_path,
      (.recall_at_10 // ""),
      (.mean_q_time // ""),
      (.truth10_in_frontier // ""),
      (.truth100_in_frontier // ""),
      (.exact_rerank // ""),
      (.dropped_before_exact // ""),
      (.mean_spearman // ""),
      (.mean_rank_shift // ""),
      (.total_storage // ""),
      (.total_storage_bytes // ""),
      "",
      ""
    ]
  | @tsv
)
