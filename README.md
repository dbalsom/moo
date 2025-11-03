![moo_logo](/img/moo_header_01.png)

## The MOO File Format

**MOO** is a simple chunked binary format used to encode x86 CPU [SingleStepTests](https://github.com/singleStepTests/).

**MOO** stands for **M**achine **O**pcode **O**peration.

## Parser implementations

- **Rust**: A Rust crate for working with MOO files, `moo-rs`, is available in [/crates/moo](/crates/moo).
- **C++**: A single-header MOO parser is available in [/cpp](/cpp)

## MOO Utilities (Mootilities?)

- A general utility for working with MOO files called `moo_util` is available
  under [/crates/moo_util](/crates/moo_util/README.md).
    - See its README for more information on how to use it.

- A python script `moo2json.py` is available under [/python](/python). This script can be used to convert a single MOO
  file or an entire set of MOO files into the more traditional SingleStepTest JSON format.

> [!NOTE]  
> If you end up writing a MOO parser in another language, or expand the capabilities of one of the parsers above,
> please consider contributing your parser via a PR so that others can benefit from your work.

## MOO Format Specification

See the documentation for the [current MOO specification, v1.1](/doc/moo_format_v1.md)
