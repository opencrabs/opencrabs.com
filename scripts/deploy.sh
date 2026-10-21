#!/usr/bin/env bash
# Deploy the OpenCrabs landing page and docs to production, in one command.
#
# Usage:
#   scripts/deploy.sh                build, push, deploy (refreshes changed cards)
#   scripts/deploy.sh --docs         rebuild the docs book first (mdbook + OG previews)
#   scripts/deploy.sh --og           force a full regeneration of the landing cards
#   scripts/deploy.sh --skip-build   deploy the image already on Docker Hub
#   scripts/deploy.sh --dry-run      print every step, change nothing
#
# This replaces the hand dance:
#   crabslandlogin -> crabslanddeploy -> ssh neuraprodnew -> cldeploy
#
# The same script runs in CI (.github/workflows/deploy.yml). There is no .env
# on a runner, so DOCKER_LOGIN_TOKEN is read from the environment instead, and
# CRABSLAND_BUILD_ARGS carries extra buildx flags (the gha layer cache).
#
# The docs ship inside the image. src/docker/Dockerfile copies the committed
# docs/book into nginx, so rebuild the book locally (--docs) whenever the docs
# changed. og_previews.py needs macOS Chrome and sips, so it can only run on
# this machine and never inside the Linux image build.
#
# The landing preview cards work the same way: they need Chrome and sips, and
# the rewritten tags plus the JPEGs are committed. CI only ever ships what is
# committed, so it cannot render them. What CI does instead is refuse to ship
# a stale card: the --check passes below need no browser and fail the deploy
# when a card no longer matches its page copy or the latest release.
set -euo pipefail

IMAGE="opencrabs/landing:latest"
DOCKERFILE="src/docker/Dockerfile"
REGISTRY_USER="opencrabs"
SERVER="${CRABSLAND_SERVER:-neuraprodnew}"
SERVER_DIR="/srv/rs/opencrabs.com"
SITE="https://opencrabs.com/"
DOCS_CHECK="https://docs.opencrabs.com/channels/telegram"
CHROME="${CHROME_BIN:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}"

# The exact steps the cldeploy alias on neuraprodnew runs, expanded because a
# non-interactive ssh shell does not load /root/.bash_aliases (where both
# cldeploy and dockercleanup are defined).
REMOTE_CMDS="cd ${SERVER_DIR} && git pull --ff-only && cd src/docker && docker compose pull && docker compose down && docker builder prune -a --force && docker network prune --force && docker image prune -a --force && docker compose up -d"

BUILD=1
DOCS=0
OG=0
DRY_RUN=0

while [ $# -gt 0 ]; do
    case "$1" in
        --docs) DOCS=1 ;;
        --og) OG=1 ;;
        --skip-build) BUILD=0 ;;
        --dry-run) DRY_RUN=1 ;;
        -h|--help) sed -n '2,/^$/p' "$0"; exit 0 ;;
        *) echo "deploy: unknown option '$1'" >&2; exit 2 ;;
    esac
    shift
done

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

step() { printf '\n==> %s\n' "$*"; }

run() {
    local cmd="$1"
    printf '+ %s\n' "$cmd"
    if [ "$DRY_RUN" = 0 ]; then
        eval "$cmd"
    fi
}

step "Preflight"
for bin in docker ssh curl; do
    command -v "$bin" >/dev/null || { echo "deploy: $bin is not on PATH" >&2; exit 1; }
done
[ -f "$DOCKERFILE" ] || { echo "deploy: $DOCKERFILE is missing" >&2; exit 1; }
echo "repo:   $REPO_ROOT"
echo "commit: $(git rev-parse --short HEAD) $(git log -1 --pretty=%s)"
if [ -n "$(git status --porcelain --untracked-files=no)" ]; then
    echo "note:   uncommitted tracked changes are part of this build:"
    git status --short --untracked-files=no
fi

if [ "$DOCS" = 1 ]; then
    step "Rebuilding the docs book (mdbook + per-page OG previews)"
    run "bash docs/build-all.sh"
fi

step "Open Graph previews"
# Render only the cards whose copy, logo, template or release changed. A host
# without Chrome cannot render, so it goes straight to the checks, which fail
# the deploy instead of shipping something stale.
if [ -x "$CHROME" ]; then
    if [ "$OG" = 1 ]; then
        run "python3 scripts/og_landing.py"
    else
        run "python3 scripts/og_landing.py --changed"
    fi
else
    echo "note: headless Chrome is not on this host, so cards cannot be rendered here"
    echo "      the checks below still run and stop the deploy if anything is stale"
fi

if [ "$DRY_RUN" = 1 ]; then
    echo "+ python3 scripts/og_landing.py --check"
    echo "+ python3 docs/scripts/og_previews.py --check"
else
    python3 scripts/og_landing.py --check || {
        echo "deploy: the landing previews are stale, run: python3 scripts/og_landing.py" >&2
        exit 1
    }
    python3 docs/scripts/og_previews.py --check || {
        echo "deploy: the docs previews are stale, run: bash docs/build-all.sh" >&2
        exit 1
    }
fi

step "Docker Hub login as $REGISTRY_USER"
# Locally the token lives in .env; in CI it arrives through the environment.
if [ -f .env ]; then
    set -a
    # shellcheck disable=SC1091
    . ./.env
    set +a
fi
[ -n "${DOCKER_LOGIN_TOKEN:-}" ] || { echo "deploy: DOCKER_LOGIN_TOKEN is missing (set it in .env or the environment)" >&2; exit 1; }
if [ "$DRY_RUN" = 0 ]; then
    printf '%s' "$DOCKER_LOGIN_TOKEN" | docker login -u "$REGISTRY_USER" --password-stdin
else
    echo "+ docker login -u $REGISTRY_USER --password-stdin  (token from .env or the environment)"
fi

if [ "$BUILD" = 1 ]; then
    step "Building and pushing $IMAGE for linux/amd64"
    run "docker buildx build --platform linux/amd64 --provenance=false --push -t $IMAGE -f $DOCKERFILE ${CRABSLAND_BUILD_ARGS:-} ."
else
    step "Skipping the build, deploying $IMAGE as it sits on Docker Hub"
fi

step "Deploying on $SERVER"
if [ "$DRY_RUN" = 0 ]; then
    # shellcheck disable=SC2029  # the steps are meant to be expanded here
    ssh "$SERVER" "$REMOTE_CMDS"
else
    echo "+ ssh $SERVER '$REMOTE_CMDS'"
fi

step "Verifying"
if [ "$DRY_RUN" = 1 ]; then
    echo "+ ssh $SERVER docker ps --filter name=opencrabs-landing"
    echo "+ curl $SITE"
    echo "+ curl $DOCS_CHECK"
    echo
    echo "dry run: nothing was built, pushed or restarted."
    exit 0
fi

ssh "$SERVER" "docker ps --filter name=opencrabs-landing --format '{{.Names}} | {{.Image}} | {{.Status}}'"

code=""
for _ in 1 2 3 4 5 6; do
    code="$(curl -sS -o /dev/null -m 20 -w '%{http_code}' "$SITE" || true)"
    [ "$code" = "200" ] && break
    sleep 5
done
echo "$SITE -> ${code:-no answer}"

og="$(curl -sS -m 20 "$DOCS_CHECK" | grep -oE '<meta property="og:image" content="[^"]*"' | head -1 || true)"
echo "docs preview: ${og:-no og:image found}"

if [ "$code" != "200" ]; then
    echo "deploy: $SITE answered ${code:-nothing} after the restart" >&2
    exit 1
fi

case "$og" in
    *"/og/"*) ;;
    *) echo "WARNING: $DOCS_CHECK still points at the site-wide preview, this image does not carry the new docs" >&2 ;;
esac

step "Done, $IMAGE is live on $SERVER"
