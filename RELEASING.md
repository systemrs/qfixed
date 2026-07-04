# Releasing qfixed

`qfixed` is published to [crates.io](https://crates.io/crates/qfixed) with an
automated [release-plz](https://release-plz.dev) pipeline that mirrors the
`systemrs/systemrs` setup: **no long-lived registry token is stored anywhere**.
Ongoing releases authenticate to crates.io with a short-lived token minted via
[Trusted Publishing](https://crates.io/docs/trusted-publishing) (GitHub OIDC), and
every publish pauses for a manual approval on a protected environment.

## How the pipeline works

Two workflows both trigger on push to `master`:

- **[`.github/workflows/release-plz-pr.yml`](.github/workflows/release-plz-pr.yml)** —
  opens/updates a **"Release" PR** that bumps the shared workspace version
  ([`[workspace.package] version`](Cargo.toml)), regenerates
  [`crates/qfixed/CHANGELOG.md`](crates/qfixed/CHANGELOG.md) from the Conventional
  Commit history, and runs `cargo-semver-checks`. It holds **no** crates.io
  credential and never publishes.
- **[`.github/workflows/release-plz.yml`](.github/workflows/release-plz.yml)** — a
  cheap `gate` job checks whether the push actually bumped the workspace version
  (i.e. the Release PR was just merged). Only then does the `publish` job run, and
  it **pauses on the protected `crates-io-publish` environment for manual
  approval**. After approval it exchanges a GitHub OIDC token for a short-lived
  crates.io token, then release-plz publishes the crate, pushes the git tag, and
  creates the GitHub release.

Both workflows mint a short-lived **GitHub App** token per run (never the default
`GITHUB_TOKEN`) so the Release PR triggers CI and the App — not the publish
credential — is what pushes tags and opens PRs.

## One-time setup (bootstrap)

You need: ownership of the `qfixed` name on crates.io (i.e. be the first to publish
it) and admin on the `systemrs/qfixed` GitHub repo. **Do steps 1–6 before merging
the branch that adds these workflows to `master`** (that first push is what starts
the automation).

### 1. Generate a one-time crates.io API token

crates.io → **Account Settings → API Tokens → New Token**:

- **Scopes:** `publish-new` (this is a brand-new crate). You may add
  `publish-update` too.
- **Expiry:** short (e.g. 7 days) — you will revoke it in step 4 anyway.
- **Crate scope:** restrict to `qfixed` if the UI offers it.

Copy the token; it is shown only once.

### 2. Bootstrap-publish `0.1.0` locally (reserves the name)

A Trusted Publisher can't be attached until the crate exists on crates.io, so the
first publish is manual. From a clean checkout of this branch:

```sh
# dry run first — should package + build with no errors
cargo publish -p qfixed --dry-run

# real publish, passing the token via env (keeps it out of ~/.cargo/credentials)
CARGO_REGISTRY_TOKEN=<token-from-step-1> cargo publish -p qfixed
```

Confirm <https://crates.io/crates/qfixed> now shows `0.1.0`.

### 3. Configure Trusted Publishing on crates.io

crates.io → **qfixed → Settings → Trusted Publishing → Add a new publisher
(GitHub)**:

| Field | Value |
|---|---|
| Repository owner | `systemrs` |
| Repository name | `qfixed` |
| Workflow filename | `release-plz.yml` |
| Environment | `crates-io-publish` |

The environment name **must** match `environment: crates-io-publish` in
`release-plz.yml`.

### 4. Revoke the bootstrap token

Back in crates.io API Tokens, **revoke** the token from step 1 — OIDC handles every
future publish, so no static registry token should linger.

### 5. Set up the `release-plz` GitHub App

Reuse the `systemrs` org's existing `release-plz` App if there is one, or create a
new App (GitHub → **Settings → Developer settings → GitHub Apps → New**):

- **Repository permissions:** `Contents` = **Read & write**, `Pull requests` =
  **Read & write**. (These are the only scopes the App needs; it cannot publish to
  crates.io.)
- **Install** the App on the `systemrs/qfixed` repository.
- Generate a **private key** (downloads a `.pem`).

Then in **`systemrs/qfixed` → Settings → Secrets and variables → Actions**:

- **Variables → New:** `RELEASE_PLZ_APP_CLIENT_ID` = the App's *Client ID*.
- **Secrets → New:** `RELEASE_PLZ_APP_KEY` = the full contents of the `.pem`.

### 6. Create the protected `crates-io-publish` environment

`systemrs/qfixed` → **Settings → Environments → New environment** →
`crates-io-publish`:

- Add **Required reviewers** (yourself). This is the human gate the publish job
  waits on — protect that GitHub login with a passkey / hardware key.

No "Allow GitHub Actions to create and approve pull requests" org toggle is needed:
the App token (not the default `GITHUB_TOKEN`) creates the Release PR.

### 7. Merge to `master`

Merge the branch that adds this pipeline. See the **first-merge note** below for the
one expected quirk.

## Normal release flow (after bootstrap)

1. Land Conventional-Commit PRs (`feat:`, `fix:`, `feat!:`…) on `master`.
   `just commit-lint` checks the format locally.
2. release-plz opens/updates the **"Release" PR** — review the proposed version bump
   and changelog.
3. **Merge the Release PR.**
4. Open the triggered **Release** workflow run and **approve** the `crates-io-publish`
   deployment (the passkey-gated approval).
5. release-plz publishes to crates.io, pushes the `qfixed-vX.Y.Z` tag, and creates the
   GitHub release.

## Notes & follow-ups

- **First-merge quirk (expected, harmless):** the commit that first introduces
  `version = "0.1.0"` into the root `Cargo.toml` makes the `gate` job see
  `previous=<none>, current=0.1.0` → `release=true`, so the publish job will request
  approval on that first merge. Since `0.1.0` is already bootstrap-published, either
  **don't approve** it, or approve and let release-plz no-op (it skips versions
  already on crates.io).
- **Hardening follow-up — pin actions by SHA:** the release workflows pin third-party
  actions by movable tag (`@v3`, `@v0.5`, …), matching systemrs. Because the publish
  job holds `id-token: write` and the App token, pinning each action to a full commit
  SHA (with the version as a trailing comment) closes a supply-chain vector where a
  compromised/force-moved tag runs before the approval gate. If you adopt it, add
  Dependabot/Renovate (SHA-pinning mode) so the pins still receive security updates.
- **Semver baseline:** `cargo-semver-checks` compares against the last crates.io
  release; before `0.1.0` exists it simply skips, and it engages from the next release
  onward.
