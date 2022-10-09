# Static site generator
Implemented using Rust (actually not implemented at all).

```mermaid
    flowchart LR;
    A[content in Markdown]-->B[static site generator];
    C[HTML templates]-->B;
    B-->D[index.html];
    B-->E[PDF files];
```

# CI/CD
Implemented using Gihub workflows feature.
Build stages:

```mermaid
    flowchart LR;
    A[provision VM/container]-->B[install Rust toolchains];
    B-->C[checkout repository];
    C-->D[build static pages];
    C-->E[site availability test];
    D-->F[pack artifacts];
    F-->G[deploy artifacts];
```

# render HTML into PDF
In order to use `wkhtmltopdf` crate eponymous utility should be
installed on a building platform.

## Arch Linux
```bash
sudo pacman -S wkhtmltopdf
```

## Ubuntu
```bash
wget https://github.com/wkhtmltopdf/packaging/releases/download/0.12.6-1/wkhtmltox_0.12.6-1.focal_amd64.deb
sudo apt install ./wkhtmltox_0.12.6-1.focal_amd64.deb
```
