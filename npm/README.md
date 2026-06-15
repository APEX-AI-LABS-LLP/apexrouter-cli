# npm distribution for apexrouter-cli

Publishes the `apexrouter-cli` binary to npm so the **ApexRouter** product line can
ship the right platform binary with zero friction:

- **AionCLI** (the Node CLI) declares it as a dependency — npm resolves the one
  matching binary automatically.
- **ApexRouter desktop** (Electron) resolves it from `node_modules` in
  `app/scripts/prepareApexRouterCore.js` (the script's documented path 0) instead
  of hand-placing or downloading-by-tag.
- **End users** can `npx @APEX-AI-LABS-LLP/apexrouter-cli …` or `npm i -g`.

## Layout — launcher + per-platform packages (the esbuild/Biome pattern)

One launcher package + six binary packages, each gated by `os`/`cpu`:

| Package | os / cpu | contains |
|---|---|---|
| `@APEX-AI-LABS-LLP/apexrouter-cli` | — | `index.js` (`binaryPath()`), `bin/apexrouter-cli.js` shim, **optionalDependencies** on all six below |
| `@APEX-AI-LABS-LLP/apexrouter-cli-darwin-arm64` | darwin / arm64 | `bin/apexrouter-cli` |
| `@APEX-AI-LABS-LLP/apexrouter-cli-darwin-x64` | darwin / x64 | `bin/apexrouter-cli` |
| `@APEX-AI-LABS-LLP/apexrouter-cli-linux-arm64` | linux / arm64 | `bin/apexrouter-cli` |
| `@APEX-AI-LABS-LLP/apexrouter-cli-linux-x64` | linux / x64 | `bin/apexrouter-cli` |
| `@APEX-AI-LABS-LLP/apexrouter-cli-win32-arm64` | win32 / arm64 | `bin/apexrouter-cli.exe` |
| `@APEX-AI-LABS-LLP/apexrouter-cli-win32-x64` | win32 / x64 | `bin/apexrouter-cli.exe` |

Because each platform package declares `os`/`cpu`, npm installs **only the one**
matching the consumer's machine (the other five are skipped as optional deps).
The `<os>-<cpu>` keys are exactly node's `${process.platform}-${process.arch}` —
the same key the desktop uses for `bundled-apexrouter-cli/<key>/`.

> **Linux is glibc** (`*-unknown-linux-gnu`), matching the desktop's
> AppImage/deb/rpm targets and AionCLI's (non-Docker) audience. No musl.

## How consumers use it

```js
// AionCLI / any Node host: spawn the engine directly.
const { binaryPath } = require("@APEX-AI-LABS-LLP/apexrouter-cli");
const { spawn } = require("node:child_process");
const child = spawn(binaryPath(), ["--json-stream", "--provider", "anthropic"], {
  stdio: ["pipe", "pipe", "inherit"],
});
```

Desktop (`prepareApexRouterCore.js`, cross-arch builds): install the **named**
platform package for the *target* arch — do **not** rely on `os`/`cpu`
auto-resolution, which keys off the *build host* and would put the wrong arch in
a cross-built installer:

```bash
npm install @APEX-AI-LABS-LLP/apexrouter-cli-darwin-x64@<version> --no-save
# then copy node_modules/@APEX-AI-LABS-LLP/apexrouter-cli-darwin-x64/bin/apexrouter-cli
# into resources/bundled-apexrouter-cli/darwin-x64/
```

## How it's built & published

`.github/workflows/release.yml` already cross-builds the six targets and uploads
them as release assets. The `publish-npm` job (gated on `post-tag-smoke`, so npm
only serves binaries that passed `--version` on their native OS):

1. downloads the six release archives,
2. extracts each to `binaries/<rust-triple>/apexrouter-cli[.exe]`,
3. runs `node npm/generate.mjs --version <v> --binaries binaries --out npm-dist`,
4. `npm publish`es the six platform packages first, then the launcher.

### Prerequisites (one-time)

- Create the **`@APEX-AI-LABS-LLP` npm org** (or claim the scope).
- Add an **`NPM_TOKEN`** automation token as a repo/org secret. Until it exists,
  the `publish-npm` job no-ops with a notice rather than failing the release.
- Optional: enable npm **provenance** by adding `permissions: { id-token: write }`
  to the job and `--provenance` to the publish step.

## Local verification

The generator is pure Node (no deps). Smoke it with a fake binary:

```bash
T=/tmp/wcore-npm-test; mkdir -p "$T/binaries/aarch64-apple-darwin"
printf '#!/bin/sh\necho "apexrouter-cli $*"\n' > "$T/binaries/aarch64-apple-darwin/apexrouter-cli"
chmod +x "$T/binaries/aarch64-apple-darwin/apexrouter-cli"
node npm/generate.mjs --version 0.0.0 --binaries "$T/binaries" --out "$T/dist" --allow-missing
# Then symlink the two packages into a node_modules and run the bin shim.
```

`--allow-missing` lets a partial set publish locally; CI runs **without** it so a
missing platform fails the release loudly.
