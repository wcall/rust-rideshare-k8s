# Rust Rideshare on Kubernetes (Grafana Cloud)

Kubernetes manifests that run the Pyroscope Rust **rideshare** example — three
regional rideshare apps plus a load generator — and ship profiles directly to a
Grafana Cloud Pyroscope stack.

This folder is self-contained: it includes the deployment manifests, the
Dockerfiles, and the application source needed to build the images. The source
originates from the upstream Pyroscope repo at
[`examples/language-sdk-instrumentation/rust/rideshare`](https://github.com/grafana/pyroscope/tree/main/examples/language-sdk-instrumentation/rust/rideshare).

## Folder structure

```
rust-rideshare-k8s/
├── README.md                 # This file.
├── kustomization.yaml        # Kustomize entrypoint; builds the credentials Secret from .env.
├── namespace.yaml            # The `rideshare` namespace.
├── rideshare.yaml            # A Deployment + Service per region (us-east, eu-north, ap-south).
├── load-generator.yaml       # Deployment that drives traffic at the regional Services.
├── Dockerfile                # Builds the rideshare app image (rideshare-rust).
├── Dockerfile.load-generator # Builds the load generator image.
├── .dockerignore             # Excludes server/target/ from build contexts.
├── server/                   # Rust rideshare app crate (Cargo.toml, Cargo.lock, src/).
├── load-generator.py         # Load generator script (used by Dockerfile.load-generator).
├── .gitignore                # Keeps .env out of version control.
├── .env.example              # Template for your Grafana Cloud connection settings.
└── .env                      # Your real settings (created from .env.example; do not commit).
```

The three rideshare Deployments share one image and the
`grafana-cloud-credentials` Secret (via `envFrom`), differing only by their
`REGION` env var. Each is fronted by a Service named after its region, so the
load generator reaches them at `http://<region>:5000`.

## Prerequisites

- A Kubernetes cluster (`kind`, `minikube`, or any cluster) and `kubectl`.
- `kustomize` (or `kubectl` v1.14+, which has it built in via `-k`).
- A Grafana Cloud stack with Pyroscope.

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

Load them into your local cluster (skip if you pushed to a registry your
cluster can pull from):

```bash
# kind
kind load docker-image rideshare-rust:latest rideshare-rust-load-generator:latest

# minikube
minikube image load rideshare-rust:latest
minikube image load rideshare-rust-load-generator:latest
```

If you push to a registry instead, update the `image:` fields in
`rideshare.yaml` and `load-generator.yaml` accordingly.

## 2. Configure Grafana Cloud credentials

From this folder:

```bash
cp .env.example .env
# edit .env with your stack endpoint, instance ID, and access policy token
```

`kustomize` turns `.env` into the `grafana-cloud-credentials` Secret, which the
app Deployments consume via `envFrom`. Keep `.env` out of version control so
your token is never committed.

## 3. Deploy

```bash
kubectl apply -k .
```

## 4. Verify

```bash
kubectl -n rideshare get pods
kubectl -n rideshare logs deploy/load-generator
```

Then open your Grafana Cloud stack and find the `rust-ride-sharing-app`
application in the Profiles app, broken down by the `region` tag.

## Tear down

```bash
kubectl delete -k .
```
