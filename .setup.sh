#!/bin/sh
set -eu

ORANGE="\033[33m"
RED="\033[31m"
RESET="\033[0m"

log() {
  printf "%b\n" "$1"
}

is_make_available() {
  if ! command -v make >/dev/null; then
    log "${RED}error: make is not available.$RESET\nPlease install ${ORANGE}make$RESET for your OS and run this script again."
    return 1
  fi
}

is_crate_bin_locally_available() {
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
  if ! is_crate_bin_locally_available "$crate"; then
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

write_pre_commit_hook() {
  echo "make fmt" >.git/hooks/pre-commit
  chmod +x .git/hooks/pre-commit
  log "  .git/hooks/pre-commit (formatting)"
}

write_commit_msg_hook() {
  cat >.git/hooks/commit-msg <<'EOF'
#!/bin/sh
alias convco=.cargo/bin/convco

# https://convco.github.io/check/
z40=0000000000000000000000000000000000000000

main() {
  if ! cat .git/COMMIT_EDITMSG | convco check --from-stdin --ignore-reverts;then
    printf "%s\n" "Please refer to https://www.conventionalcommits.org/en/v1.0.0"
    exit 1
  fi
}

main
EOF

  chmod +x .git/hooks/commit-msg
  log "  .git/hooks/commit-msg (conventional commits linting)"
}

write_hooks() {
  log "Writing hooks..."
  write_pre_commit_hook
  write_commit_msg_hook
}

find_editor() {
  editor=""
  global_git_editor=$(git config --global --get core.editor || true)

  if [ -n "$global_git_editor" ]; then
    editor="$global_git_editor"
  elif [ -n "$EDITOR" ]; then
    editor="$EDITOR"
  elif command -v nvim >/dev/null; then
    editor="nvim"
  elif command -v vim >/dev/null; then
    editor="vim"
  elif command -v vi >/dev/null; then
    editor="vi"
  elif command -v code >/dev/null; then
    editor="code"
  elif command -v emacs >/dev/null; then
    editor="emacs"
  elif command -v nano >/dev/null; then
    editor="nano"
  elif command -v notepad >/dev/null; then
    editor="notepad"
  fi

  echo "$editor"
}

integrate_convco_with_git() {
  git config --local core.editor ".cargo/bin/convco commit"

  # do not use convco for interactive rebase (git rebase -i)
  # use system's default editor instead
  editor=$(find_editor)

  if [ -n "$editor" ]; then
    git config --local sequence.editor "$editor"
  else
    log "${RED}error: No editor found.$RESET"
    log "Please set the ${ORANGE}EDITOR$RESET environment variable to your preferred editor and run ${ORANGE}make setup$RESET again."
  fi

  log "Integrated convco with git"
}

end_log() {
  log "===================\nTo get started, you can run the make tasks defined in the Makefile.\n"
  make help
}

main() {
  is_make_available
  install_dev_deps
  write_hooks
  integrate_convco_with_git
  end_log
}

main
