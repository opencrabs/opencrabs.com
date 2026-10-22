FROM rust:1.94-bookworm AS builder

RUN rustup target add wasm32-unknown-unknown

# Use pre-built binaries to avoid dependency hell
RUN curl -fsSL https://github.com/trunk-rs/trunk/releases/download/v0.21.7/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xz -C /usr/local/bin

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY locales/ locales/
COPY src/ src/
COPY index.html style.css Trunk.toml ./
COPY chrome/ chrome/
COPY docs/ docs/

RUN trunk build --release

# The public usage dashboard is a separate crate with its own Cargo.lock. It is
# static output only: the page fetches /stats-data/public.json, which the host
# writes every five minutes and compose mounts in read-only.
COPY stats/Cargo.toml stats/Cargo.lock stats/Trunk.toml stats/index.html stats/stats.css /app/stats/
COPY stats/src/ /app/stats/src/
RUN cd /app/stats && trunk build --release

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY --from=builder /app/stats/dist /usr/share/nginx/html/stats
COPY --from=builder /app/src/root-files/ /usr/share/nginx/html/
COPY --from=builder /app/docs/book /usr/share/nginx/html/docs
COPY src/docker/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 19283
