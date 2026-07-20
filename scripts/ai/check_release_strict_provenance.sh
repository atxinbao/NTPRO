#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

MANIFEST="${NTPRO_V33_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_33_0_release_manifest.json}"
OUTPUT="${NTPRO_V33_STRICT_OUTPUT:-target/ntpro-v330/v0_33_0_strict_release_manifest.json}"

scripts/ai/check_backend_maintenance_release.sh

mkdir -p "$(dirname "$OUTPUT")"
files_json="$(
  while IFS= read -r path; do
    digest="$(shasum -a 256 "$path" | awk '{print $1}')"
    jq -n --arg path "$path" --arg sha256 "$digest" \
      '{path: $path, sha256: $sha256}'
  done < <(jq -r '.release_inputs[]' "$MANIFEST") | jq -s .
)"

jq -nS \
  --arg schema_version "ntpro.v330_strict_release_provenance.v1" \
  --arg release_version "v0.33.0" \
  --arg release_tag "ntpro-rust-only-v0.33.0" \
  --arg source_commit "$(git rev-parse HEAD)" \
  --arg audit_strategy "source_tree_plus_github_remote" \
  --argjson files "$files_json" \
  '{
    schema_version: $schema_version,
    release_version: $release_version,
    release_tag: $release_tag,
    source_commit: $source_commit,
    audit_strategy: $audit_strategy,
    local_generated_evidence_required_in_source_tree: false,
    remote_reconstruction_required: true,
    files: $files
  }' >"$OUTPUT"

jq -e '
  .schema_version == "ntpro.v330_strict_release_provenance.v1"
  and .release_version == "v0.33.0"
  and .release_tag == "ntpro-rust-only-v0.33.0"
  and .audit_strategy == "source_tree_plus_github_remote"
  and .local_generated_evidence_required_in_source_tree == false
  and .remote_reconstruction_required == true
  and (.files | length == 5)
  and (.files | all(
    (.path | type == "string" and length > 0)
    and (.sha256 | test("^[0-9a-f]{64}$"))
  ))
' "$OUTPUT" >/dev/null

echo "v33_strict_provenance=pass files=5 output=$OUTPUT"
