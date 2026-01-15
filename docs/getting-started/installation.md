# Installation

## Requirements

- Python 3.9 or higher
- A supported platform (Linux, macOS, Windows)

## Installing with pip

```bash
pip install transit-parser
```

## Installing with uv

```bash
uv add transit-parser
```

## Installing from source

Clone the repository and install with maturin:

```bash
git clone https://github.com/example/transit-parser
cd transit-parser
pip install maturin
maturin develop --release
```

## Optional Dependencies

### pandas support

For DataFrame output:

```bash
pip install transit-parser[pandas]
# or
pip install pandas
```

## Verifying Installation

```python
import transit_parser
print(transit_parser.__version__)
# Output: 0.1.0
```
