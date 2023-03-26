# nhsf

```
nhfs is a file server that serve static file and provide a way to configure the directory html rendering

Usage: nhsf [OPTIONS]

Options:
  -c, --conf <CONF>  [default: /etc/nhfs/config.toml]
  -h, --help         Print help
  -V, --version      Print version
```

## Installation

1.  From Sources
    -   Clone the repository:
        ```sh
        git clone https://github.com/nxthat/nhfs
        ```
    -   Build for production
        ```sh
        cargo build --release
        ```


## Configuration

To configure `nhsf` you need to create a `yaml` file, let considere this `nhsf.conf`:

```yaml
host: http://0.0.0.0:8080
path: ./conf_dir
directory: ./dir_exposed
```

```

```

We recommand to use docker to deploy `nhsf`


```sh
docker run  ghcr.io/nxthat/nhsf:0.0.1
```
