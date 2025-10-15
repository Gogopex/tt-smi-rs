# tt-smi vs tt-smi-rs comparison

## Test Environment

- **System**: TT-QuietBox (WH)

## How to Run Comparisons

### Prerequisites
1. Ensure both `tt-smi` and `tt-smi-rs` are available
3. Tenstorrent hardware properly installed and accessible

### Step 1: Generate Python Snapshot
```bash
cd /path/to/tt-metal
source python_env/bin/activate

cd /path/to/tt-smi
python -m tt_smi -f /tmp/python_snapshot.json
```

### Step 2: Generate Rust Snapshot
```bash
cd /path/to/tt-smi-rs
cargo build --release

./target/release/tt-smi-rs snapshot -o /tmp/rust_snapshot.json
```

### Step 3: Compare Snapshots
Use the provided comparison script:

```bash
python compare_snapshots.py /tmp/python_snapshot.json /tmp/rust_snapshot.json
```

