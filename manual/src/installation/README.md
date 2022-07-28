# Installation

In theory, one should be able to just install the rust-analyzer binary and have it automatically work with any editor. We are not there yet, so some editor specific setup is required.

Additionally, rust-analyzer needs the sources of the standard library. If the source code is not present, rust-analyzer will attempt to install it automatically.

To add the sources manually, run the following command:

```shell
rustup component add rust-src
```
