# prs docker image

## Build

```bash
cd ./pkg/docker

docker build -t prs:latest .
```

## Run

```bash
docker run --rm -it -v ~/:/root prs:latest
docker run --rm -it -v ~/:/root prs:latest help
```

In your shell you might alias this to:

```bash
alias prs='docker run --rm -it -v ~/:/root prs:latest'
```
