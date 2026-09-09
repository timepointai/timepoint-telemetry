# Generated media in Clockchain

Consumer contract addition authorized by Sean on 2026-09-08. This does not
change TT's envelope, ontology, bundle, content hash or classification rules.

Generated images are measured provenance attached to a particular claim body.
They are not historical evidence, and never participate in identity, dedup,
near-match or feasibility. The existing historical claim is not rewritten.

Clockchain uses a separate `cc.image-attachment.v1` manifest. It binds
`source_body_hash`, `source_entity_id`, the original PNG's `image_sha256` and
`byte_count`, generator model/revision, prompt, seed, generation timestamp,
and license provenance. Its `kind` is `generated_interpretation_of_claim`;
`historical_verification` is `not_assessed`. A distinct media-writer signature
authenticates this manifest. It does not masquerade as a historical attestation
or claim event, and is not covered by the historical ledger's existing anchors.

Admission verifies the signature, object digest, PNG format, declared size and
current projection of the source body. Read responses distinguish current and
stale source bindings. An image is never silently reassigned after a correction.
Missing media is `no_image`, not evidence that generation was requested or failed.
Generation jobs, if introduced, must expose their own queued/failed states.

Image bytes live outside the claim/projection tables in a content-addressed
object directory or object store. Losing those bytes is an availability failure,
not a reason to modify historical identity. The manifest remains independently
verifiable and exports retain provenance and license conditions.

Stability's SDXL 1.0 local checkpoint can be used under CreativeML Open RAIL++-M,
including its synthetic-output derivative-training provisions. This is conditional
permission: use restrictions and derivative-model distribution obligations remain.
Do not label it unrestricted. Hosted API permission is a separate assessment.
The model license, checkpoint revision and weight hashes must be recorded.

Source: https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/blob/462165984030d82259a11f4367a4eed129e94a7b/LICENSE.md
