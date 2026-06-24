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
  if ($r.kind == "load" and $r.values.phase == "total") then
    .load_seconds = $r.values.seconds
  elif ($r.kind == "load" and $r.values.phase == "build_index") then
    .build_index_seconds = $r.values.seconds
  elif ($r.kind == "recall" and $r.values.ef_search == "200") then
    .recall_at_10 = $r.values["recall@k"]
    | .recall_mean_q_time = $r.values["mean q-time"]
  elif $r.kind == "hnsw-frontier" then
    .truth10_in_emitted_pool = $r.values["truth@10 in emitted pool"]
    | .truth100_in_emitted_pool = $r.values["truth@100 in emitted pool"]
    | .emitted_pool = $r.values["emitted pool"]
    | .exact_rerank = $r.values["exact rerank"]
    | .quantized_rerank = $r.values["quantized rerank"]
    | .pool_dropped_before_exact = $r.values["pool dropped before exact"]
  elif $r.kind == "hnsw-score-correlation" then
    .mean_spearman = $r.values["mean spearman"]
    | .mean_rank_shift = $r.values["mean |rank shift|"]
    | .exact_top4_max_approx_rank = $r.values["exact top4 max approx rank"]
  elif ($r.kind == "storage" and $r.metric == "storage_field" and $r.values.field == "total") then
    .total_storage = $r.values.value
    | .total_storage_bytes = $r.values.value_bytes
  else
    .
  end;

def complete_enough_for_table:
  .scale and .format and .build_path and .recall_at_10;

([
  "Scale",
  "Format",
  "Build path",
  "Load seconds",
  "Build-index seconds",
  "Recall@10 ef=200",
  "Recall mean q-time ef=200",
  "Truth@10 in emitted pool",
  "Truth@100 in emitted pool",
  "Emitted pool",
  "Exact rerank",
  "Quantized rerank",
  "Pool dropped before exact",
  "Mean Spearman",
  "Mean rank shift",
  "Exact top4 max approx rank",
  "Total storage",
  "Total storage bytes"
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
      (.load_seconds // ""),
      (.build_index_seconds // ""),
      (.recall_at_10 // ""),
      (.recall_mean_q_time // ""),
      (.truth10_in_emitted_pool // ""),
      (.truth100_in_emitted_pool // ""),
      (.emitted_pool // ""),
      (.exact_rerank // ""),
      (.quantized_rerank // ""),
      (.pool_dropped_before_exact // ""),
      (.mean_spearman // ""),
      (.mean_rank_shift // ""),
      (.exact_top4_max_approx_rank // ""),
      (.total_storage // ""),
      (.total_storage_bytes // "")
    ]
  | @tsv
)
