/*
    MOO-rs Copyright 2025 Daniel Balsom
    https://github.com/dbalsom/moo

    Permission is hereby granted, free of charge, to any person obtaining a
    copy of this software and associated documentation files (the “Software”),
    to deal in the Software without restriction, including without limitation
    the rights to use, copy, modify, merge, publish, distribute, sublicense,
    and/or sell copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
    FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.
*/

use chrono::Local;
use clap::Parser;
use flate2::read::GzDecoder;
use plotly::{common::Title, layout::Layout, Bar, Pie, Plot, Table};

use plotly::{
    common::{color::Color as PlotlyColor, Font},
    traces::table::{Cells, Header},
};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use serde::Serialize;

// ---- import your MOO types (adjust to your module layout as needed) ----
use moo::{
    prelude::*,
    types::{flags::MooCpuFlag, MooRegister},
};

#[derive(Clone, Debug, Serialize)]
struct ColorGrid(Vec<Vec<String>>);
impl PlotlyColor for ColorGrid {}

/// Generate an HTML report from a directory of binary MOO files.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Input directory containing *.moo or *.moo.gz
    input_dir: PathBuf,

    /// Output HTML (default: ./moo_report.html)
    #[arg(short, long, default_value = "moo_report.html")]
    output: PathBuf,

    /// Recurse into subdirectories
    #[arg(short = 'r', long)]
    recursive: bool,
}

fn mnemonic_to_string(bytes: [u8; 8]) -> String {
    let s = bytes
        .iter()
        .take_while(|&&c| c != 0) // stop at first NUL
        .map(|&c| c as char)
        .collect::<String>();
    s.trim_end().to_string()
}

fn flags_to_string(flags: &[MooCpuFlag]) -> String {
    let mut flag_set = HashSet::new();
    for flag in flags {
        flag_set.insert(flag);
    }

    // format as odiszapc string.
    let o_chr = if flag_set.contains(&MooCpuFlag::OF) { 'o' } else { '.' };
    let d_chr = if flag_set.contains(&MooCpuFlag::DF) { 'd' } else { '.' };
    let i_chr = if flag_set.contains(&MooCpuFlag::IF) { 'i' } else { '.' };
    let s_chr = if flag_set.contains(&MooCpuFlag::SF) { 's' } else { '.' };
    let z_chr = if flag_set.contains(&MooCpuFlag::ZF) { 'z' } else { '.' };
    let a_chr = if flag_set.contains(&MooCpuFlag::AF) { 'a' } else { '.' };
    let p_chr = if flag_set.contains(&MooCpuFlag::PF) { 'p' } else { '.' };
    let c_chr = if flag_set.contains(&MooCpuFlag::CF) { 'c' } else { '.' };

    format!(
        "{}{}{}{}{}{}{}{}",
        o_chr, d_chr, i_chr, s_chr, z_chr, a_chr, p_chr, c_chr
    )
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    env_logger::init();

    // 1) gather files
    let files = collect_moo_files(&args.input_dir, args.recursive)?;
    if files.is_empty() {
        fs::write(&args.output, empty_report_html(&args.input_dir))?;
        println!("No MOO files found; wrote {}", args.output.display());
        return Ok(());
    }

    // 2) read & calc stats
    let mut rows = Vec::new();
    for path in files {
        match load_moo_file(&path) {
            Ok(mut tf) => {
                let mnemonic = if let Some(metadata) = tf.metadata() {
                    mnemonic_to_string(metadata.mnemonic)
                }
                else {
                    "<unknown>".to_string()
                };

                let s = tf.calc_stats();
                rows.push(FileRow::from_stats(path, mnemonic, s));
            }
            Err(e) => {
                eprintln!("Failed to read {}: {e}", path.display());
            }
        }
    }

    if rows.is_empty() {
        fs::write(&args.output, empty_report_html(&args.input_dir))?;
        println!("All reads failed; wrote {}", args.output.display());
        return Ok(());
    }

    // 3) plots
    let table_plot = build_table_plot(&rows)?;
    let (_ops_pie, cycles_bar) = build_summary_plots(&rows)?;
    let dual_pies = build_dual_pies(&rows)?;

    // 4) glue into one HTML
    let html = compose_html_report(
        &args.input_dir,
        &[
            ("files_table", table_plot),
            ("dual_pies", dual_pies),
            ("cycles_bar", cycles_bar),
        ],
    );

    // 5) write
    fs::write(&args.output, html)?;
    println!("Report written to {}", args.output.display());
    Ok(())
}

#[derive(Debug, Clone)]
struct FileRow {
    file_name: String,
    mnemonic: String,
    regs_modified: Vec<String>,
    total_cycles: usize,
    min_cycles: usize,
    max_cycles: usize,
    avg_cycles: f64,
    mem_reads: usize,
    mem_writes: usize,
    code_fetches: usize,
    io_reads: usize,
    io_writes: usize,
    wait_states: usize,
    flags_modified: String,
    flags_always_set: String,
    flags_always_cleared: String,
    exceptions_seen: Vec<u8>,
    exceptions_hist: Vec<(u8, usize)>, // NEW: [(exception, count)] sorted by exception
    exceptions_total: usize,           // NEW: total occurrences for percentage calc
    total_tests: usize,
}

impl FileRow {
    fn from_stats(path: PathBuf, mnemonic: String, s: MooTestFileStats) -> Self {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<unknown>")
            .to_string();

        // histogram for percentages
        let mut hist: HashMap<u8, usize> = HashMap::new();
        for e in &s.exceptions_seen {
            if *e < 32 {
                *hist.entry(*e).or_insert(0) += 1;
            }
        }
        let exceptions_total = s.exceptions_seen.len();
        let mut exceptions_hist: Vec<(u8, usize)> = hist.into_iter().collect();
        exceptions_hist.sort_unstable_by_key(|(k, _)| *k);

        // display list: dedup + sort
        let mut exceptions_seen: Vec<u8> = s.exceptions_seen;
        exceptions_seen.sort_unstable();
        exceptions_seen.dedup();

        let mut regs_modified = s.registers_modified.clone();
        regs_modified.retain(|r| !matches!(r, MooRegister::EFLAGS | MooRegister::EIP));

        Self {
            file_name,
            mnemonic,
            regs_modified: regs_modified.iter().map(|r| format!("{r:?}")).collect(),
            total_cycles: s.total_cycles,
            min_cycles: s.min_cycles,
            max_cycles: s.max_cycles,
            avg_cycles: s.avg_cycles,
            mem_reads: s.mem_reads,
            mem_writes: s.mem_writes,
            code_fetches: s.code_fetches,
            io_reads: s.io_reads,
            io_writes: s.io_writes,
            wait_states: s.wait_states,
            flags_modified: flags_to_string(&s.flags_modified),
            flags_always_set: flags_to_string(&s.flags_always_set),
            flags_always_cleared: flags_to_string(&s.flags_always_cleared),
            exceptions_seen,
            exceptions_hist,
            exceptions_total,
            total_tests: s.test_count,
        }
    }
}

/// Recursively (or not) collect *.moo and *.moo.gz files
fn collect_moo_files(dir: &Path, recursive: bool) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if recursive {
        for e in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if e.file_type().is_file() && is_moo_path(e.path()) {
                out.push(e.path().to_path_buf());
            }
        }
    }
    else {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_file() && is_moo_path(&path) {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

fn is_moo_path(p: &Path) -> bool {
    let ext = p.extension().and_then(|s| s.to_str()).map(|s| s.to_ascii_lowercase());
    if ext.as_deref() == Some("moo") {
        return true;
    }
    if ext.as_deref() == Some("gz") {
        // accept *.moo.gz (case-insensitive)
        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
            return stem.to_ascii_lowercase().ends_with(".moo");
        }
    }
    false
}

/// Load a MooTestFile from a binary (optionally gzipped) file.
fn load_moo_file(path: &Path) -> anyhow::Result<MooTestFile> {
    let bytes = if path.extension().and_then(|s| s.to_str()) == Some("gz") {
        let f = File::open(path)?;
        let mut gz = GzDecoder::new(f);
        let mut buf = Vec::new();
        gz.read_to_end(&mut buf)?;
        buf
    }
    else {
        fs::read(path)?
    };

    let mf = MooTestFile::read(&mut std::io::Cursor::new(bytes))?;
    Ok(mf)
}

/// Estimate column widths from content lengths (roughly 7 px per char),
/// clamped to [min_px, max_px] and padded a bit.
fn estimate_column_widths(cols: &[Vec<String>], min_px: f64, max_px: f64, pad_px: f64) -> Vec<f64> {
    let char_px = 7.0; // conservative average; Plotly uses a sans-serif default
    cols.iter()
        .map(|col| {
            let max_chars = col.iter().map(|s| s.len()).max().unwrap_or(0) as f64;
            let w = max_chars * char_px + pad_px;
            w.clamp(min_px, max_px)
        })
        .collect()
}

fn build_table_plot(rows: &[FileRow]) -> anyhow::Result<Plot> {
    let file_names: Vec<String> = rows.iter().map(|r| r.file_name.clone()).collect();
    let mnemonics: Vec<String> = rows.iter().map(|r| r.mnemonic.clone()).collect();
    let regs_modified: Vec<String> = rows.iter().map(|r| r.regs_modified.join(", ")).collect();
    let total_cycles: Vec<String> = rows.iter().map(|r| r.total_cycles.to_string()).collect();
    let min_cycles: Vec<String> = rows.iter().map(|r| r.min_cycles.to_string()).collect();
    let max_cycles: Vec<String> = rows.iter().map(|r| r.max_cycles.to_string()).collect();
    let avg_cycles: Vec<String> = rows.iter().map(|r| format!("{:.2}", r.avg_cycles)).collect();
    let mem_reads: Vec<String> = rows.iter().map(|r| r.mem_reads.to_string()).collect();
    let mem_writes: Vec<String> = rows.iter().map(|r| r.mem_writes.to_string()).collect();
    let code_fetches: Vec<String> = rows.iter().map(|r| r.code_fetches.to_string()).collect();
    let io_reads: Vec<String> = rows.iter().map(|r| r.io_reads.to_string()).collect();
    let io_writes: Vec<String> = rows.iter().map(|r| r.io_writes.to_string()).collect();
    //let waits: Vec<String> = rows.iter().map(|r| r.wait_states.to_string()).collect();
    let flags_modified: Vec<String> = rows.iter().map(|r| r.flags_modified.clone()).collect();
    let flags_always_set: Vec<String> = rows.iter().map(|r| r.flags_always_set.clone()).collect();
    let flags_always_cleared: Vec<String> = rows.iter().map(|r| r.flags_always_cleared.clone()).collect();

    let excs: Vec<String> = rows
        .iter()
        .map(|r| {
            if r.exceptions_total == 0 {
                "-".to_string()
            }
            else {
                r.exceptions_hist
                    .iter()
                    .map(|(code, count)| {
                        let pct = (*count as f64) * 100.0 / (r.exceptions_total as f64);
                        format!("{code} ({pct:.0}%)")
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        })
        .collect();

    // per-test % for the total column
    let exc_totals: Vec<String> = rows
        .iter()
        .map(|r| {
            if r.total_tests == 0 {
                r.exceptions_total.to_string()
            }
            else {
                let pct = (r.exceptions_total as f64) * 100.0 / (r.total_tests as f64);
                format!("{} ({:.1}%)", r.exceptions_total, pct)
            }
        })
        .collect();

    // --- header (unchanged except columns) ---
    let header = Header::new(vec![
        "file",
        "mnemonic",
        "regs mod",
        "total cyc",
        "min cyc",
        "max cyc",
        "avg cyc",
        "mem reads",
        "mem writes",
        "code fetches",
        "io reads",
        "io writes",
        "f modified",
        "f always set",
        "f always clr",
        "exceptions",
        "exc_total",
    ])
    .fill(Fill::new().color("rgba(230,230,230,1.0)"))
    .font(Font::new().color("black").size(14)); // black text, bigger font

    // --- cells data ---
    let cols: Vec<Vec<String>> = vec![
        file_names,
        mnemonics,
        regs_modified,
        total_cycles,
        min_cycles,
        max_cycles,
        avg_cycles,
        mem_reads,
        mem_writes,
        code_fetches,
        io_reads,
        io_writes,
        flags_modified,
        flags_always_set,
        flags_always_cleared,
        excs,
        exc_totals,
    ];

    let row_colors: Vec<String> = rows
        .iter()
        .map(|r| {
            let pct = if r.total_tests == 0 {
                0.0
            }
            else {
                (r.exceptions_total as f64) * 100.0 / (r.total_tests as f64)
            };
            if pct > 33.33 {
                "rgba(255,210,210,1)".to_string() // light pink
            }
            else {
                "rgba(255,255,255,1)".to_string() // white
            }
        })
        .collect();

    let num_columns = cols.len();
    let fill_grid: Vec<Vec<String>> = (0..num_columns).map(|_| row_colors.clone()).collect();

    use plotly::traces::table::Fill;
    let cells = Cells::new(cols).fill(Fill::new().color(ColorGrid(fill_grid)));

    // .align(
    //         vec!["left", "left", "right","right","right","right",
    //                     "right","right","right","right","right","right",
    //                     "left","right"]); // optional: pack numbers tighter

    // --- plot/table trace ---
    let mut plot = Plot::new();
    let table = Table::new(header, cells).name("Per-file stats").column_width(10.0);
    plot.add_trace(table);

    plot.set_layout(
        Layout::new()
            .title(Title::with_text("MOO Report — Per-file Statistics"))
            .auto_size(true)
            .height(900),
    );
    Ok(plot)
}

fn build_exceptions_pie(rows: &[FileRow]) -> anyhow::Result<Plot> {
    use std::collections::HashMap;

    let mut totals: HashMap<u8, usize> = HashMap::new();
    let mut grand_total: usize = 0;

    for r in rows {
        for (code, count) in &r.exceptions_hist {
            if *code < 32 {
                *totals.entry(*code).or_insert(0) += *count;
                grand_total += *count;
            }
        }
    }

    let (labels, values): (Vec<String>, Vec<f64>) = if grand_total == 0 {
        (vec!["none".to_string()], vec![1.0])
    }
    else {
        let mut pairs: Vec<(u8, usize)> = totals.into_iter().collect();
        pairs.sort_unstable_by_key(|(code, _)| *code);

        let labels = pairs
            .iter()
            .map(|(code, _)| format!("INT {}", code))
            .collect::<Vec<_>>();
        let values = pairs.iter().map(|(_, ct)| *ct as f64).collect::<Vec<_>>();

        (labels, values)
    };

    let mut plot = Plot::new();
    let pie = Pie::new(values).labels(labels).name("Exceptions (overall)");
    plot.add_trace(pie);
    plot.set_layout(
        Layout::new()
            .title(Title::with_text("Overall Exceptions by Type"))
            .auto_size(true),
    );

    Ok(plot)
}

fn build_dual_pies(rows: &[FileRow]) -> anyhow::Result<Plot> {
    // --- Operation mix (as before) ---
    let (reads, writes, fetches, io_r, io_w, waits) = rows.iter().fold((0, 0, 0, 0, 0, 0), |acc, r| {
        (
            acc.0 + r.mem_reads,
            acc.1 + r.mem_writes,
            acc.2 + r.code_fetches,
            acc.3 + r.io_reads,
            acc.4 + r.io_writes,
            acc.5 + r.wait_states,
        )
    });
    let op_labels = vec![
        "Mem Reads",
        "Mem Writes",
        "Code Fetches",
        "IO Reads",
        "IO Writes",
        "Wait States",
    ];
    let op_values = vec![reads, writes, fetches, io_r, io_w, waits]
        .into_iter()
        .map(|v| v as f64)
        .collect::<Vec<_>>();

    let op_pie = Pie::new(op_values)
        .labels(op_labels)
        .name("Operation Mix")
        .domain(plotly::common::Domain::new().x(&[0.0, 0.48]).y(&[0.0, 1.0]));

    // --- Exceptions pie (<32 only) ---
    use std::collections::HashMap;
    let mut totals: HashMap<u8, usize> = HashMap::new();
    let mut grand_total = 0;
    for r in rows {
        for (code, count) in &r.exceptions_hist {
            if *code < 32 {
                *totals.entry(*code).or_insert(0) += *count;
                grand_total += *count;
            }
        }
    }
    let (exc_labels, exc_values): (Vec<String>, Vec<f64>) = if grand_total == 0 {
        (vec!["none".into()], vec![1.0])
    }
    else {
        let mut pairs: Vec<(u8, usize)> = totals.into_iter().collect();
        pairs.sort_unstable_by_key(|(c, _)| *c);
        (
            pairs.iter().map(|(c, _)| format!("INT {}", c)).collect(),
            pairs.iter().map(|(_, ct)| *ct as f64).collect(),
        )
    };
    let exc_pie = Pie::new(exc_values)
        .labels(exc_labels)
        .name("Exceptions")
        .domain(plotly::common::Domain::new().x(&[0.52, 1.0]).y(&[0.0, 1.0]));

    // --- Combined plot ---
    let mut plot = Plot::new();
    plot.add_trace(op_pie);
    plot.add_trace(exc_pie);
    plot.set_layout(
        Layout::new()
            .title(Title::with_text("Operation Mix vs Exceptions"))
            .auto_size(true)
            .height(500),
    );
    Ok(plot)
}

/// Build overall operation-mix pie + per-file cycles bar.
fn build_summary_plots(rows: &[FileRow]) -> anyhow::Result<(Plot, Plot)> {
    let (reads, writes, fetches, io_r, io_w, waits) = rows.iter().fold((0, 0, 0, 0, 0, 0), |acc, r| {
        (
            acc.0 + r.mem_reads,
            acc.1 + r.mem_writes,
            acc.2 + r.code_fetches,
            acc.3 + r.io_reads,
            acc.4 + r.io_writes,
            acc.5 + r.wait_states,
        )
    });

    // pie chart
    let labels = vec![
        "Mem Reads",
        "Mem Writes",
        "Code Fetches",
        "IO Reads",
        "IO Writes",
        "Wait States",
    ];
    let values = vec![reads, writes, fetches, io_r, io_w, waits]
        .into_iter()
        .map(|v| v as f64)
        .collect::<Vec<_>>();

    let mut pie_plot = Plot::new();
    let pie = Pie::new(values).labels(labels).name("Operation Mix");
    pie_plot.add_trace(pie);
    pie_plot.set_layout(
        Layout::new()
            .title(Title::with_text("Overall Operation Mix"))
            .auto_size(true),
    );

    // bar chart: total cycles per file
    let x = rows.iter().map(|r| r.file_name.clone()).collect::<Vec<_>>();
    let y = rows.iter().map(|r| r.total_cycles as f64).collect::<Vec<_>>();
    let mut bar_plot = Plot::new();
    let bar = Bar::new(x, y).name("Total Cycles");
    bar_plot.add_trace(bar);
    bar_plot.set_layout(
        Layout::new()
            .title(Title::with_text("Total Cycles per File"))
            .auto_size(true),
    );

    Ok((pie_plot, bar_plot))
}

/// Compose one HTML page with all figures via Plotly CDN.
fn compose_html_report(input_dir: &Path, figures: &[(&str, Plot)]) -> String {
    let now = Local::now();
    let heading = format!(
        "MOO Report &mdash; {}<br><small>Source directory: {}</small>",
        now.format("%Y-%m-%d %H:%M:%S"),
        input_dir.display()
    );

    let mut divs_and_scripts = String::new();
    for (i, (id, plot)) in figures.iter().enumerate() {
        let div_id = format!("{}_{}", id, i);
        let json = plot.to_json();
        divs_and_scripts.push_str(&format!(
            r#"<div id="{div_id}" class="plot-wrap"></div>
<script>(function(){{
  var fig = {json};
  // make sure layout is autosized (in case a trace didn't set it)
  if (!fig.layout) fig.layout = {{}};
  fig.layout.autosize = true;

  // merge any existing config with responsive:true
  var cfg = Object.assign({{responsive:true}}, fig.config || {{}});
  Plotly.newPlot('{div_id}', fig.data, fig.layout, cfg);
}})();</script>
"#,
        ));
    }

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>MOO Report</title>
<script src="https://cdn.plot.ly/plotly-2.35.2.min.js"></script>
<style>
body {{
  font-family: system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif;
  margin: 24px;
  background: #0f1115;
  color: #e6e6e6;
}}
h1 {{ font-weight: 700; font-size: 20px; margin: 0 0 16px 0; }}
.card {{
  background: #151923; border-radius: 12px; padding: 16px 20px;
  box-shadow: 0 0 0 1px #242b3a inset;
}}
hr {{ border: none; border-top: 1px solid #242b3a; margin: 24px 0; }}
.small {{ color: #9aa2b2; }}
</style>
</head>
<body>
  <div class="card">
    <h1>{heading}</h1>
    <div class="small">Generated by moo-report</div>
  </div>
  <hr/>
  {divs_and_scripts}
</body>
</html>"#,
        heading = heading,
        divs_and_scripts = divs_and_scripts
    )
}

/// Tiny HTML if no files found
fn empty_report_html(input_dir: &Path) -> String {
    format!(
        r#"<!doctype html>
<html><head><meta charset="utf-8"/><title>MOO Report</title>
<style>body{{font-family:system-ui;margin:24px}}</style></head>
<body>
<h1>No MOO files found</h1>
<p>Searched: <code>{}</code></p>
<p>Expected <code>.moo</code> or <code>.moo.gz</code>.</p>
</body></html>"#,
        input_dir.display()
    )
}
