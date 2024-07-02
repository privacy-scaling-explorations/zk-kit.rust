#!/bin/sh
set -eou pipefail

ORANGE="\033[33m"
RESET="\033[0m"

log() {
  printf "%b\n" "$1"
}

is_bin_locally_available() {
  crate="$1"
  [ -x ".cargo/bin/$crate" ]
}

install_local() {
  crate="$1"
  log "Installing $ORANGE$crate$RESET locally..."
  cargo install --root .cargo "$crate"
}

maybe_install_local() {
  crate="$1"
  if ! is_bin_locally_available "$crate"; then
    install_local "$crate"
  else
    log "  $ORANGE$crate$RESET already installed locally. Skipping."
  fi
}

install_dev_deps() {
  log "Installing development dependencies..."
  crates="convco dprint cargo-nextest"

  for crate in $crates; do
    maybe_install_local "$crate"
  done
}

write_pre_push_hook() {
  cat >.git/hooks/pre-push <<'EOF'
#!/bin/sh
alias convco=.cargo/bin/convco

# https://convco.github.io/check/
z40=0000000000000000000000000000000000000000

while read -r _ local_sha _ remote_sha; do
  if [ "$local_sha" != $z40 ]; then
    if [ "$remote_sha" = $z40 ]; then
      # New branch, examine all commits
      range="$local_sha"
    else
      # Update to existing branch, examine new commits
      range="$remote_sha..$local_sha"
    fi

    # Check only the commits that are not in main
    merge_base=$(git merge-base "$local_sha" main)
    if [ -n $merge_base ];then
      range="$merge_base..$local_sha"
    fi

    # Check for WIP commit
    if ! convco check "$range"; then
      exit 1
    fi
  fi
done
EOF

  chmod +x .git/hooks/pre-push
  log "  .git/hooks/pre-push (conventional commits linting)"
}

write_pre_commit_hook() {
  echo "make fmt" >.git/hooks/pre-commit
  chmod +x .git/hooks/pre-commit
  log "  .git/hooks/pre-commit (formatting)"
}

write_hooks() {
  log "Writing hooks..."
  write_pre_commit_hook
  write_pre_push_hook
}

end_log() {
  log "===================\nTo get started, you can run the make tasks defined in the Makefile.\n"
  make help
}

main() {
  install_dev_deps
  write_hooks
  end_log
}

main
