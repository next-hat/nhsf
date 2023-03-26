<h1 style="text-align: center">
nhsf
</h1>

```
nhfs is a server that serve a static directory and its subdirectories with templating

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
        cd nhfs
        ```
    -   Build for production
        ```sh
        cargo build --release
        ```

    -   Run the binary:
        ```sh
        ./target/release/nhfs -c /path/to/my/config.yml
        ```

2.  Using docker
    -   Get the image
        ```sh
        docker pull ghcr.io/nxthat/nhfs:0.1.0
        ```
    -   Run the image:
        ```
        docker run -v /etc/nhsf:/etc/nhsf ghcr.io/nxthat/nhfs:0.1.0
        ```


## Configuration

To configure `nhsf` you need to create a `yaml` file, let considere this `nhsf.conf`:

```yaml
# The address to listen to
host: http://0.0.0.0:8080
# The path of the directory where your templates are
path: ./conf_dirory
# The directory to expose
directory: ./dir_exposed
```

```
nhfs -c nhsf.conf
```

See the [example](/example/) directory for in deep understanding
