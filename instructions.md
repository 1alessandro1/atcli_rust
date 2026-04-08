## Build the optimized static binary
```bash
RUSTFLAGS="-C target-feature=+crt-static" cargo build --target armv7-unknown-linux-gnueabihf --release
```

**3. Extreme Size Optimization (Optional but recommended):**
Embedded filesystems have extremely limited space. Compress the binary down to ~150-200KB using `strip` and `upx`:
```bash
arm-linux-gnueabihf-strip --strip-all target/armv7-unknown-linux-gnueabihf/release/atcmd
upx --best --lzma target/armv7-unknown-linux-gnueabihf/release/atcmd
```

## Usage 

**Default mode (defaults to `/dev/smd11`):**
```bash
./atcmd ati
```

**Custom port mode:**
```bash
./atcmd -p /dev/ttyUSB0 at+cpin?
```

**Help:**
```bash
./atcmd -h
```
