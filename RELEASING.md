# Releasing apexrouter-cli

This repo publishes pre-built binaries to GitHub Releases for the ApexRouter app
(`scripts/prepareApexRouterCore.js`) to download. Releases are normally produced
by CI; this doc covers both the happy path and the manual fallback.

## Versioning

`Cargo.toml` `[workspace.package].version` is the source of truth.
Bumps are driven by [release-please](https://github.com/googleapis/release-please)
from conventional-commit messages on `main`:

- `feat(...): ...` → minor bump (pre-1.0: patch bump per
  `bump-patch-for-minor-pre-major: true`)
- `fix(...): ...` → patch bump
- `feat!: ...` or `BREAKING CHANGE:` footer → major bump (post-1.0)

`chore`, `ci`, `docs`, `style`, `test`, `build` types do **not** bump.

## Happy path — release via CI

1. Merge work into `main` using conventional-commit messages.
2. `release-please` opens a "Release" PR titled
   `chore(main): release X.Y.Z`. The PR updates `CHANGELOG.md`,
   `Cargo.toml` version, and `.release-please-manifest.json`.
3. Review the PR. When the version bump and changelog look right, merge it.
4. On merge, the `Release Please` workflow:
   - Creates git tag `vX.Y.Z` and a GitHub Release with auto-generated notes.
   - Calls the `Release` workflow via `workflow_call`.
5. The `Release` workflow builds `apexrouter-cli` for six targets, packages each
   as `apexrouter-cli-vX.Y.Z-<target>.{tar.gz,zip}`, generates
   `apexrouter-cli-checksums.txt`, mints a **keyless Sigstore build-provenance
   attestation** for each archive (`actions/attest-build-provenance`), and
   uploads all artifacts to the GitHub Release created in step 4.
6. The app's `scripts/prepareApexRouterCore.js` downloads the asset matching its
   host platform from `https://github.com/APEX-AI-LABS-LLP/apexrouter-cli/releases/`.
7. `publish-npm` publishes `@APEX-AI-LABS-LLP/apexrouter-cli` (+ platform packages)
   with `npm publish --provenance`, emitting a transparency-logged provenance
   statement per package.

## Signing & provenance (keyless)

There is **no long-lived release signing key to manage.** Both distribution
channels are signed keylessly via GitHub OIDC + Sigstore:

- **GitHub release archives** — `actions/attest-build-provenance` binds each
  archive to the workflow that built it (SLSA provenance, logged in the public
  Sigstore transparency log). Requires `id-token: write` + `attestations: write`
  on the `github-release` job (already set). Public repo only.
- **npm packages** — `npm publish --provenance` under `id-token: write`. The
  package `repository.url` **must** case-match the GitHub slug
  (`APEX-AI-LABS-LLP/apexrouter-cli`); npm 422s on a mismatch (enforced in
  `npm/generate.mjs`).

`apexrouter-cli self-update` verifies the archive's attestation with
`gh attestation verify` before installing, and **fails closed** if `gh` is
absent (it does not skip verification). There is nothing to rotate; deleting or
rolling a key is not part of a release cut.

Targets built:

| OS      | Arch    | Rust target                  |
|---------|---------|------------------------------|
| Linux   | x86_64  | `x86_64-unknown-linux-gnu`   |
| Linux   | aarch64 | `aarch64-unknown-linux-gnu`  (cross) |
| macOS   | x86_64  | `x86_64-apple-darwin`        |
| macOS   | aarch64 | `aarch64-apple-darwin`       |
| Windows | x86_64  | `x86_64-pc-windows-msvc`     |
| Windows | aarch64 | `aarch64-pc-windows-msvc`    |

## Manual dispatch (CI is green but you want to re-run packaging)

```bash
gh workflow run release.yml \
  --repo APEX-AI-LABS-LLP/apexrouter-cli \
  --field tag_name=vX.Y.Z
```

The tag must already exist. Re-runs upload with `--clobber` and replace
prior assets on the same release.

## Manual fallback — CI broken, tag already cut

If the `Release` workflow fails partway and you need binaries before the fix
lands, build locally and upload by hand.

Per target, on the matching host (or via `cross` for Linux aarch64):

```bash
git checkout vX.Y.Z
cargo build --release --target <target> -p wcore-cli
cd target/<target>/release
tar -czf apexrouter-cli-vX.Y.Z-<target>.tar.gz apexrouter-cli   # or apexrouter-cli.exe on Windows (use zip there)
```

Then:

```bash
gh release upload vX.Y.Z \
  apexrouter-cli-vX.Y.Z-<target>.tar.gz \
  --repo APEX-AI-LABS-LLP/apexrouter-cli \
  --clobber
```

Regenerate checksums after all six artifacts are uploaded:

```bash
shasum -a 256 apexrouter-cli-vX.Y.Z-* > apexrouter-cli-checksums.txt
gh release upload vX.Y.Z apexrouter-cli-checksums.txt --clobber
```

## Verifying a release

After publication, smoke-check the asset list:

```bash
gh release view vX.Y.Z --repo APEX-AI-LABS-LLP/apexrouter-cli --json assets \
  --jq '.assets[].name'
```

Expect six platform archives plus `apexrouter-cli-checksums.txt`.
