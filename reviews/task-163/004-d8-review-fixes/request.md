# Review request — Task 163 D8 review fixes

**Status:** review requested; code follow-up only, not Task 163 closeout.

**Branch:** `task-179-ec-distann-physical-shards`  
**Code checkpoint:** `de9d6fca3e0bd05f44ad6b0d376a2480e4023798`

## Outcome

This checkpoint handles the implementation findings from packet 003 without
overstating the remaining measurement gate. Completed shard graphs are encoded
inside their scoped workers and drained to backend-owned `BufFile` storage as
they arrive; assignment membership lists are consumed before stitch, and the
cursor merge retains only bounded cursor plus one-group state.

## Changes under review

- Replace batch-wide `Vec<ShardGraph>` completion collection with worker-side
  flat encoding and a bounded completion channel. All PostgreSQL temporary-file
  I/O remains on the backend thread.
- Consume assignment membership lists during shard construction and preserve
  only scalar diagnostics into stitch.
- Poll PostgreSQL interrupts from the backend during assignment, completion
  waits, stitch groups, and reachability repair.
- Validate physical EOF after the declared entry stream and make `BufFile`
  cleanup safe during unwind.
- Reuse centroid/read scratch buffers and use checked, conservative accounting
  for cursors, opaque `BufFile` overhead, scratch, and transient `Vec` growth.
- Add six corrupt-spool negative tests and one PG18 fixture whose mandatory
  headers alone force at least one real `BufFile` across a block boundary.

## Exact-SHA validation

The packet artifacts were produced from a clean detached checkout at the code
checkpoint, with a shared target directory used only to avoid rebuilding
unchanged dependencies.

- Focused shard suite: 16 passed, 0 failed.
- Strict PG18 library clippy with `-D warnings`: clean.
- Focused live PG18 multi-block test: 1 passed, 0 failed.

## Deliberately open

Task 163 still needs `ecaz bench suite` evidence for the new NOTICE fields and
peak RSS at 10k/50k/100k, plus an old-vs-new 10k graph-digest or recall A/B.
This request does not claim that measurement condition or Task 163 closeout.

## Reviewer focus

1. Verify the completion channel removes retained completed graph batches
   without allowing PostgreSQL APIs onto worker threads.
2. Verify all paths drain workers safely on error/interrupt and resource-owner
   cleanup is sound during unwind.
3. Verify cursor scratch reuse and transient-allocation accounting are
   conservative.
4. Verify the corrupt-spool matrix and runtime multi-block proof cover the
   packet 003 conditions while leaving benchmark evidence open.
