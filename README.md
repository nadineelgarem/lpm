# Linux Process Manager (LPM)

A Rust-based **Linux Process Manager** with both **Command Line Interface (CLI)** and **Graphical User Interface (GUI)**.

This project was built by **Nadine Elgarem**, together with her colleagues, to provide a powerful tool for monitoring and managing system processes.

---

##  Features

###  CLI Features
- List running processes
- Filter processes by name (`--filter`)
- Filter processes by user (`--user`)
- Sort processes by CPU (`--sort cpu`) or memory usage (`--sort mem`)
- Kill a process by PID (`--kill`)
- Restart a process by PID (`--restart`)
- Export process list to JSON (`--export`)
- Check alerts for high CPU or RAM usage (`--alerts`)
- View history of actions taken (`--history`)

###  GUI Features
- Display live list of processes in a table
- Filter by process name or user
- Sort by CPU, memory, or user
- Kill or restart processes from the interface
- Export process data to JSON
- Check and display system alerts
- View process action history in a dedicated tab

---

##  Tools & Crates Used

- [`sysinfo`](https://crates.io/crates/sysinfo) → Retrieve system and process info
- [`clap`](https://crates.io/crates/clap) → Parse CLI arguments and commands
- [`gtk4`](https://crates.io/crates/gtk4) → Build the graphical user interface
- [`chrono`](https://crates.io/crates/chrono) → Handle timestamps for process history

---


