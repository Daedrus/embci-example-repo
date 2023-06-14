I am using a Raspberry Pi Pico board, the documentation for it (and the RP2040
processor) is [here](https://www.raspberrypi.com/documentation/microcontrollers/raspberry-pi-pico.html).

The RP2040 is a Dual-core Arm Cortex M0+ processor. Details about the various
Arm Cores can be found [here](https://en.wikipedia.org/wiki/ARM_Cortex-M).

---

Currently running a minimal Rust example that just toggles a GPIO pin. Using a
HAL right from the start abstracts away too many details, so for now only use
the `cortex-m`, `cortex-m-rt` and `panic-halt` crates. I also need the
`rp2040-boot2` crate for a bootloader that sets up the external flash memory.
Links to those crates' documentation can be found in the `Cargo.toml` file.

The command `cargo tree` can show the crates that those crates depend on:
```
~ embci-example-repo git:(main) cargo tree --format "{p} {f}"
embci-example-repo v0.1.0 (/home/embci/git/embci-example-repo)
├── cortex-m v0.7.7
│   ├── bare-metal v0.2.5 const-fn
│   │   [build-dependencies]
│   │   └── rustc_version v0.2.3
│   │       └── semver v0.9.0 default
│   │           └── semver-parser v0.7.0
│   ├── bitfield v0.13.2
│   ├── embedded-hal v0.2.7
│   │   ├── nb v0.1.3
│   │   │   └── nb v1.1.0
│   │   └── void v1.0.2
│   └── volatile-register v0.2.1
│       └── vcell v0.1.3
├── cortex-m-rt v0.7.3
│   └── cortex-m-rt-macros v0.7.0 (proc-macro)
│       ├── proc-macro2 v1.0.60 default,proc-macro
│       │   └── unicode-ident v1.0.9
│       ├── quote v1.0.28 default,proc-macro
│       │   └── proc-macro2 v1.0.60 default,proc-macro (*)
│       └── syn v1.0.109 clone-impls,default,derive,extra-traits,full,parsing,printing,proc-macro,quote
│           ├── proc-macro2 v1.0.60 default,proc-macro (*)
│           ├── quote v1.0.28 default,proc-macro (*)
│           └── unicode-ident v1.0.9
├── panic-halt v0.2.0
├── rp2040-boot2 v0.3.0
│   [build-dependencies]
│   └── crc-any v2.4.3 alloc,debug-helper,default
│       └── debug-helper v0.3.13
└── rp2040-pac v0.4.0
    ├── cortex-m v0.7.7  (*)
    └── vcell v0.1.3
```

---

The build process is controlled through the `.cargo/config` file where you can
specify the target plus `codegen` flags. Possible flags are documented
[here](https://doc.rust-lang.org/rustc/codegen-options/index.html#link-args).

The target triple for the Cortex M0+ is `thumbv6m-none-eabi` as can be seen
[here](https://doc.rust-lang.org/nightly/rustc/platform-support.html). A good
explanation on target triples can be found [here](https://www.flother.is/til/llvm-target-triple/).

The `-Tlink.x` flag currently tells the Rust linker (which by default seems
to be set to LLVM's [LLD linker](https://lld.llvm.org/), but it can be changed)
to use `link.x` as a linker script. That linker script is provided by the
`cortex-m-rt` crate which in turn includes the user-defined `memory.x` script
which is present in this repository. For more information, look at the `link.x`
script directly.

---

`rp2040-pac` (where pac stands for Peripheral Access Crate, see documentation
[here](https://docs.rust-embedded.org/discovery/microbit/04-meet-your-hardware/terminology.html#peripheral-access-crate-pac))
is the first layer of abstraction over the registers of the chip. I have only
taken a brief look at the crate's code but I feel like the keys to the kingdom
are in this crate, in understanding the types it declares and how those types
manage to express (and hide) things like "write value x to memory mapped
register y", "toggle bit a in memory mapped register y", and so on.

The crate itself is (mostly) auto-generated from what I understand, the
generation process being reproducible through the `update.sh` script
[here](https://github.com/rp-rs/rp2040-pac/blob/92f047d61fe9197fcb567127681b83e4efc6b444/update.sh).

The theory is that silicon vendors using ARM architectures in their chip
design should provide a `CMSIS-SVD` file which is an .xml file describing
everything in the chip, with the memory-mapped registers being an important
part. So the crate has used such an .svd file as its foundation, it is
included in the repository [here](https://github.com/rp-rs/rp2040-pac/blob/92f047d61fe9197fcb567127681b83e4efc6b444/svd/rp2040.svd).

However, vendors often make mistakes in these .svd files (this case being no
exception) and it seems to be such a reoccuring pattern that there exists an
entire Python library meant to assist in patching these files: [svdtools](https://pypi.org/project/svdtools/).
The library uses as input (besides the .svd file) a .yaml file where one can
specify exactly what should be changed/patched in the input .svd file.

[A patched rp2040.svd file](https://github.com/rp-rs/rp2040-pac/blob/92f047d61fe9197fcb567127681b83e4efc6b444/svd/rp2040.svd.patched)
is thus passed to [svd2rust](https://github.com/rust-embedded/svd2rust) to
produce the crate's Rust code, which is what we'll be digging into.

The file containing all the type and trait definitions (which are then used
in every other file) is [generic.rs](https://github.com/rp-rs/rp2040-pac/blob/92f047d61fe9197fcb567127681b83e4efc6b444/src/generic.rs)
and it is where our journey of understanding begins.

The Rust concepts which are immediately visible in that file and which we should
understand are all under the [generics chapter](https://doc.rust-lang.org/rust-by-example/generics.html)
of the [Rust by Example book](https://doc.rust-lang.org/rust-by-example/index.html).

TO BE CONTINUED
