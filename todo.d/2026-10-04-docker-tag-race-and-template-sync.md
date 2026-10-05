- [ ] **TODO-DOCKER-MULTIARCH** Give the Docker release a single multi-arch
      manifest. Today `release-docker.yml` runs one matrix leg per platform, and
      every leg pushes the same tags (`pr-N`, branch, `latest`, semver), so
      whichever leg finishes last owns the tag: a tag can point at an
      arm64-only image. The fix is to push each leg by digest and add a merge
      job (`docker buildx imagetools create`) that publishes the tags once. The
      security job already uses the primary (amd64) leg's digest (#33), so it is
      not affected.
- [ ] **TODO-TEMPLATE-SYNC** Check whether `.github/linters/*` and the
      `release-*.yml` / `pr-automation.yml` workflows are synced from a shared
      template. If they are, port #33's fixes upstream, or the next sync
      reverts them: the super-linter `*_CONFIG_FILE` paths, the mixed
      `VALIDATE_*` flags, `.hadolint.yaml`, the docker digest handoff and
      `id-token: write`, and the aarch64 linker in `.cargo/config.toml`.
