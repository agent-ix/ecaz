# Task 184 packet 004 artifact manifest

- Task bucket / packet: `reviews/task-184/004-full-scale-decision/`
- Candidate extension build: `765f28a548b194f1bd1ba845ae06b2266d04153b`
- Runner correctness checkpoint: `b51b0ad4795290731bf1b8044117701af6527c8a`
- Status: full-scale suite configured; evidence pending

The checked-in `full-scale-suite.json` measures one eager and one lazy10 arm on
the same immutable physical generation at each of 10k, 50k, and 100k. No
absolute latency threshold is imposed. The final decision will compare recall,
mean and tail latency, stage/work attribution, bytes, storage, construction,
topology, engagement, query separation, and installed release provenance.
