# SDX55/SDX65/SDX75 AT Command CLI (atcmd-rs)

A modern, memory-safe, and highly optimized Rust rewrite of the `atcli` and `atcmd` utilities for Qualcomm SDX55/X65/X75 modems (originally developed by Compal).

This project is the result of deep reverse engineering of the original OEM binaries. Our goal was to understand their internal mechanics, expose their critical vulnerabilities, and provide a standalone, static executable suitable for UBIFS embedded filesystems.

## 🔍 Reverse Engineering Insights & OEM Flaws

By disassembling the original C binaries (`atcli` and `atcmd`) using IDA/Ghidra, we uncovered how the OEM utilities interacted with the modem via the `/dev/smd11` serial interface. We discovered two distinct architectures, both suffering from severe design flaws.

### Flaws in the Original `atcli` (Synchronous Version)
1. **The 4KB Global Buffer (Buffer Overflow Risk):** The original code relied on a fixed 4096-byte global accumulator buffer (`byte_2410`). It sent the AT command and used a loop with `stpcpy` to append every incoming chunk of data into this single buffer until it matched a terminator. If a command returned more than 4KB of data before hitting "OK" or "ERROR" (e.g., querying available network operators), it would overflow the buffer, potentially crashing the system.
2. **Delayed Output:** The utility waited for the entire response to be collected in the global buffer before printing anything to standard output via `puts()`.

### Flaws in the Original `atcmd` (Asynchronous Version)
1. **Stream Fragmentation Bug:** The `atcmd` variant used `select()` followed by `read()` into a buffer, and then blindly used `strstr(buffer, "OK")` to check for completion. Because serial interfaces (like `/dev/smd11`) are streams, a response like `OK\r\n` can be fragmented across two separate `read()` calls (e.g., reading `O` then `K`). The `strstr` logic fails to catch this, causing the program to hang.
2. **Hardcoded 5-Second Timebomb:** Instead of waiting dynamically for a response, the main thread spawned a background reading thread (`pthreads`), issued the AT command, and then triggered a hardcoded `usleep(5000000)` (exactly 5 seconds). If the modem took 6 seconds to process a complex command, the main thread would wake up and violently terminate the program, cutting off the response.
3. **Hardware Flow Control Hacks:** The code manually forced the RTS (Ready To Send) pin using the `TIOCMBIC` ioctl (`0x5416`). Modern Linux TTY drivers typically handle this automatically.

## 🛡️ Why this Rust Rewrite is Superior

This Rust version merges the best of both worlds while addressing all OEM architectural flaws:
* **Line-by-Line Streaming:** Instead of accumulating the whole response in a global buffer, this version uses `BufReader::read_line`. It processes and prints the output dynamically line-by-line. This means memory usage is always O(1) (only needing enough RAM for a single line), eliminating any buffer overflow risk.
* **Stream-Safe Matching:** By reading until the `\n` character, we naturally bypass the serial fragmentation bugs that plagued the `strstr()` implementation in the original C code.
* **No Threading Bloat or Arbitrary Timeouts:** We use a highly efficient synchronous blocking read. No `pthreads`, no hardcoded 5-second timeouts. The program waits exactly as long as the modem needs.
* **Dynamic Port Selection:** You can seamlessly switch between the default `/dev/smd11` and custom ports (e.g., `/dev/ttyUSB0`) using the `-p` flag.
* **Fully Static & Tiny:** Compiled with `+crt-static`, it relies on zero shared libraries from the router's OS.

## 🛠️ Build Instructions for Embedded Systems

To build this for the ARMv7 architecture (used by SDX55 modems), you need the Rust toolchain and the appropriate target. See [BUILD.md](BUILD.md) for details


## License
This project is licensed under the GNU Lesser General Public License v2.1 - see the [LICENSE](LICENSE) file for details.
