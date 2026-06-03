# Rust Rideshare on Kubernetes (Uninstrumented)

Kubernetes manifests that run the Rust **rideshare** example: three regional
rideshare apps plus a load generator. This branch is intentionally
uninstrumented and does not send profiling data anywhere.

The app still exposes the same CPU-heavy routes that are useful for profiling
demos, but profiling is not enabled by default:

- `server/Cargo.toml` does not include the `pyroscope` crate.
- `server/src/main.rs` starts only the Warp HTTP server.
- `rideshare.yaml` does not inject Grafana Cloud credentials into pods.
- `kustomization.yaml` does not generate a credentials Secret.
- `.env` and `.env.example` are retained for the optional instrumentation steps
  below.

If you plan to profile this app with an eBPF profiler (for example Grafana
Alloy) instead of the Rust SDK, see [eBPF profiling and frame pointers](#ebpf-profiling-and-frame-pointers)
below — the release profile must be built with frame pointers enabled.

The source originates from the upstream Pyroscope repo at
[`examples/language-sdk-instrumentation/rust/rideshare`](https://github.com/grafana/pyroscope/tree/main/examples/language-sdk-instrumentation/rust/rideshare).

## Folder structure

```
rust-rideshare-k8s/
├── README.md                 # This file.
├── kustomization.yaml        # Kustomize entrypoint for the uninstrumented app.
├── namespace.yaml            # The `rideshare` namespace.
├── rideshare.yaml            # A Deployment + Service per region.
├── load-generator.yaml       # Deployment that drives traffic at the regional Services.
├── Dockerfile                # Builds the rideshare app image.
├── Dockerfile.load-generator # Builds the load generator image.
├── .dockerignore             # Excludes server/target/ from build contexts.
├── server/                   # Rust rideshare app crate.
├── load-generator.py         # Load generator script.
├── .gitignore                # Keeps .env out of version control.
├── .env.example              # Optional Grafana Cloud profiling settings template.
└── .env                      # Optional real settings; do not commit.
```

The three rideshare Deployments share one image and differ only by their
`REGION` env var. Each is fronted by a Service named after its region, so the
load generator reaches them at `http://<region>:5000`.

## Prerequisites

- A Kubernetes cluster (`kind`, `minikube`, or any cluster) and `kubectl`.
- `kustomize` or `kubectl` v1.14+ with built-in `-k` support.
- Docker, or another image builder compatible with your cluster.

Grafana Cloud is not required for this uninstrumented version.

## 1. Build the images

The manifests reference local image names with `imagePullPolicy: IfNotPresent`.
The Dockerfiles and app source are in this folder, so build straight from here:

```bash
# from this folder (rust-rideshare-k8s)

# App image (uses ./Dockerfile)
docker build -t rideshare-rust:latest -f Dockerfile .

# Load generator image (uses ./Dockerfile.load-generator)
docker build -t rideshare-rust-load-generator:latest -f Dockerfile.load-generator .
```

Load them into your local cluster, unless you pushed to a registry your cluster
can pull from:

```bash
# kind
kind load docker-image rideshare-rust:latest rideshare-rust-load-generator:latest

# minikube
minikube image load rideshare-rust:latest
minikube image load rideshare-rust-load-generator:latest
```

If you push to a registry instead, update the `image:` fields in
`rideshare.yaml` and `load-generator.yaml` accordingly.

## 2. Deploy

```bash
kubectl apply -k .
```

## 3. Verify

```bash
kubectl -n rideshare get pods
kubectl -n rideshare logs deploy/load-generator
```

This version should generate traffic, but no profiles should appear in Grafana
Cloud because there is no profiler in the app and no profiling credentials are
mounted into the pods.

## eBPF profiling and frame pointers

eBPF profilers (such as the Grafana Alloy `pyroscope.ebpf` component) unwind
stacks using **frame pointers**. By default the Rust release profile omits
frame pointers as an optimization, so eBPF-collected stacks would be truncated
or unsymbolized. To get complete stacks, the app must be compiled with frame
pointers preserved.

Enable them in `server/Cargo.toml` by setting `force-frame-pointers = true` on
the release profile:

```toml
[profile.release]
# Rust requires frame pointers for eBPF profiling
force-frame-pointers = true
opt-level = 0
debug = true
rpath = true
lto = false
debug-assertions = true
codegen-units = 4
```

This repo already ships with `force-frame-pointers = true`, so the default
image is ready for eBPF profiling. If you remove or disable that flag, rebuild
the image (see [Build the images](#1-build-the-images)) before profiling so the
change takes effect.

Frame pointers are required only for eBPF-based profiling. The Rust SDK setup
in the next section captures its own stacks and does not depend on this flag.

## Turn On Grafana Cloud Profiling

Use this section when you want to turn this uninstrumented app into a fully
instrumented Grafana Cloud Profiles demo. The Rust SDK setup follows the
[Grafana Pyroscope Rust documentation](https://grafana.com/docs/pyroscope/latest/configure-client/language-sdks/rust/).

### 1. Configure credentials

Keep `.env.example` and copy it to `.env`:

```bash
cp .env.example .env
```

Edit `.env` with the values from your Grafana Cloud stack's Pyroscope details:

```dotenv
PYROSCOPE_SERVER_ADDRESS=https://profiles-prod-001.grafana.net
PYROSCOPE_BASIC_AUTH_USER=123456
PYROSCOPE_BASIC_AUTH_PASSWORD=glc_your_token_here
PYROSCOPE_APPLICATION_NAME=rust-ride-sharing-app
```

Use a Grafana Cloud access policy token with `profiles:write`. Do not commit
`.env`.

### 2. Add the Rust SDK dependency

In `server/Cargo.toml`, add the `pyroscope` dependency:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
warp = "0.3"
pyroscope = { version = "^2.0.5", features = ["backend-pprof-rs"] }
log = "0.4"
pretty_env_logger = "0.5"
chrono = "0.4"
```

Then refresh the lockfile:

```bash
cd server
cargo update
```

### 3. Start the Pyroscope agent in Rust

In `server/src/main.rs`, add these imports:

```rust
use std::sync::Arc;

use pyroscope::backend::{pprof_backend, BackendConfig, PprofConfig};
use pyroscope::pyroscope::PyroscopeAgentBuilder;
```

Change the `main` signature so startup errors can be returned:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
```

After `pretty_env_logger::init_timed();`, read the Grafana Cloud settings and
start the agent:

```rust
let server_address = std::env::var("PYROSCOPE_SERVER_ADDRESS")
    .unwrap_or_else(|_| "http://localhost:4040".to_string());
let region = std::env::var("REGION").unwrap_or_else(|_| "us-east".to_string());
let app_name = std::env::var("PYROSCOPE_APPLICATION_NAME")
    .unwrap_or_else(|_| "rust-ride-sharing-app".to_string());
let auth_user = std::env::var("PYROSCOPE_BASIC_AUTH_USER").unwrap_or_default();
let auth_password = std::env::var("PYROSCOPE_BASIC_AUTH_PASSWORD").unwrap_or_default();

let agent = PyroscopeAgentBuilder::new(
    server_address,
    app_name,
    100,
    "pyroscope-rs",
    env!("CARGO_PKG_VERSION"),
    pprof_backend(PprofConfig::default(), BackendConfig::default()),
)
.basic_auth(auth_user, auth_password)
.tags(vec![("region", &region)])
.build()?;

let agent_running = agent.start()?;
```

Wrap each vehicle route with a `vehicle` tag. For example, replace the
uninstrumented `bike` route with:

```rust
let (add_tag, remove_tag) = agent_running.tag_wrapper();
let add = Arc::new(add_tag);
let remove = Arc::new(remove_tag);

let bike = warp::path("bike").map(move || {
    add("vehicle".to_string(), "bike".to_string());
    order_bike(1);
    remove("vehicle".to_string(), "bike".to_string());

    "Bike ordered"
});
```

Repeat the same pattern for `scooter` and `car`, using their existing order
functions and tag values. End `main` with `Ok(())` after the `warp::serve(...)`
line.

### 4. Mount credentials in Kubernetes

In `kustomization.yaml`, add the Secret generator:

```yaml
secretGenerator:
  - name: grafana-cloud-credentials
    envs:
      - .env

generatorOptions:
  disableNameSuffixHash: true
```

In each app container in `rideshare.yaml`, add `envFrom` after the existing
`REGION` env var:

```yaml
envFrom:
  - secretRef:
      name: grafana-cloud-credentials
```

Add this to all three Deployments: `us-east`, `eu-north`, and `ap-south`.

### 5. Rebuild, deploy, and verify profiles

```bash
docker build -t rideshare-rust:latest -f Dockerfile .
kind load docker-image rideshare-rust:latest
kubectl apply -k .
kubectl -n rideshare rollout restart deploy/us-east deploy/eu-north deploy/ap-south
```

After the load generator sends traffic, open Grafana Cloud Profiles and look for
the `rust-ride-sharing-app` application with `region` and `vehicle` labels.

## Tear down

```bash
kubectl delete -k .
```
