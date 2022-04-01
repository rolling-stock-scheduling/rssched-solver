# Installation
- install the rust compiler rustc and the rust package manager cargo: https://www.rust-lang.org/tools/install
- from the main directory, compile and run the programm with:

```bash
cargo run --release
```

# Configuration
in src/main.rs you can specify the instance. The path should point to a directory that contains the following files:

- fahrzeuggruppen.csv
- endpunkte.csv
- wartungsfenster.csv
- relationen.csv
- kundenfahrten.csv
- SBB_leistungsketten.csv

In the parent directory there must be a config.yaml file.
