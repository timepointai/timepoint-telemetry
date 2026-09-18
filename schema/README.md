# Source attribution v1

`timepoint-source/1` separates production method, claim role, producer and evidence
lineage from grounding, confidence and time. The JSON schema is the structural
contract; shared vectors exercise the additional role/method validity rules.

Only a trusted service boundary may issue an attribution. Parsing or hashing an
import does not authenticate its author. Models cannot award source status.
Unknown versions, absent producer identities, unsupported roles, and extracted
claims without evidence references must display SOURCE UNKNOWN. Source-reported
means a document made the claim, not that the claim is true. A model's summary
remains generated inference and must retain references to its input records.

The Rust implementation and the Python TDF implementation use the same vectors.
Display implementations must reproduce those labels. Neither a confidence of 1,
a `grounded` flag nor a date can change source identity. Generated artifacts
retain generated origin through export, sharing and re-import. Mixed-origin
records must carry attribution at claim granularity; a frame-wide badge must not
promote every row to the strongest source represented.

This contract is additive. Original content hashes, provenance hashes and stored
artifacts remain unchanged when a serving correction restores missing legacy
attribution. A hash authenticates byte identity, not source truth. Legacy recovery
requires server-recorded producer metadata; otherwise attribution is unknown.
