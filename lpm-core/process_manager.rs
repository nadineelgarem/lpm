use sysinfo::{System, SystemExt, ProcessExt, UserExt, PidExt};
use libc::{setpriority, PRIO_PROCESS};

pub struct ProcessManager {
    system: System,
    history: Vec<String>,
}

impl ProcessManager {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        ProcessManager {
            system,
            history: Vec::new(),
        }
    }

    /// Returns owned process data to avoid lifetime issues
    pub fn list_processes(&mut self) -> Vec<sysinfo::Process> {
        self.system.refresh_all();
        self.system.processes().values().cloned().collect()
    }

    pub fn list_processes_by_name(&mut self, name: &str) -> Vec<sysinfo::Process> {
        self.list_processes()
            .into_iter()
            .filter(|p| p.name().contains(name))
            .collect()
    }

    pub fn list_processes_by_user(&mut self, user: &str) -> Vec<sysinfo::Process> {
        self.list_processes()
            .into_iter()
            .filter(|p| {
                p.user_id()
                    .and_then(|uid| self.system.get_user_by_id(uid))
                    .map_or(false, |u| u.name() == user)
            })
            .collect()
    }

use libc::{kill, SIGKILL}; // make sure this is at the top of the file

pub fn kill_process(&mut self, pid: usize) -> bool {
    self.system.refresh_all();

    let success = unsafe { kill(pid as i32, SIGKILL) } == 0;

    self.history.push(format!("Tried to kill PID {} -> {}", pid, success));
    success
}

    pub fn restart_process(&mut self, pid: usize) -> bool {
        self.history.push(format!("Restart requested for PID {} -> unsupported", pid));
        false
    }

    pub fn change_priority(&mut self, pid: usize, priority: i32) -> bool {
        let result = unsafe { setpriority(PRIO_PROCESS, pid as u32, priority) } == 0;
        self.history.push(format!("Priority change: PID {} -> {} -> {}", pid, priority, result));
        result
    }

    pub fn export_processes(&self, format: &str, path: &str) -> std::io::Result<()> {
        let processes: Vec<_> = self.system.processes().values()
            .map(|p| format!("{}: {}", p.pid(), p.name()))
            .collect();

        let content = if format == "json" {
            serde_json::to_string_pretty(&processes).unwrap()
        } else {
            processes.join("\n")
        };

        std::fs::write(path, content)
    }

    pub fn check_alerts(&self, cpu_threshold: f32, mem_threshold: u64) -> Vec<String> {
        self.system.processes().values()
            .filter(|p| p.cpu_usage() > cpu_threshold || p.memory() > mem_threshold)
            .map(|p| format!("ALERT: {} (PID {}) CPU: {}% MEM: {} KB", p.name(), p.pid(), p.cpu_usage(), p.memory()))
            .collect()
    }

    pub fn show_history(&self) -> String {
        if self.history.is_empty() {
            "No actions performed yet.".to_string()
        } else {
            self.history.join("\n")
        }
    }
}

