# Troubleshooting FAQ

### I see a warning "Variable `None` should have snake_case name, e.g. `none`"

rust-analyzer fails to resolve `None`, and thinks you are binding to a variable
named `None`. That's usually a sign of a corrupted sysroot. Try removing and re-installing
it: `rustup component remove rust-src` then `rustup component install rust-src`.

### VSCodium / another IDE cannot resolve cargo files with rust-analyzer

This may be due to a misconfiguration of permissions with flatpak.
This can be resolved by allowing flatpak to see the cargo binaries in the PATH with the following command:
```bash
flatpak --user override com.vscodium.codium  --env=PATH=/app/bin:/usr/bin:/home/$USER/.cargo/bin
```
This issue should be resolved in VS Code, if the issue persists with it or another IDE,
you may substite the flatpak identifier with the appropriate application (EG: `com.visualstudio.code` for VS Code)
Refer to [Issue #2873](https://github.com/rust-lang/rust-analyzer/issues/2873) and [Issue #20616](https://github.com/rust-lang/rust-analyzer/issues/20616)