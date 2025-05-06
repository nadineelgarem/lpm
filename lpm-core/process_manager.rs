use sysinfo::{System, SystemExt, ProcessExt, UserExt, PidExt};

pub struct ProcessManager {
    system: System,
}

impl ProcessManager {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        ProcessManager { system }
    }

    pub fn list_processes(&mut self) -> Vec<&sysinfo::Process> {
        self.system.refresh_all();
        self.system.processes().values().collect()
    }

    pub fn list_processes_by_name(&mut self, name: &str) -> Vec<&sysinfo::Process> {
        self.list_processes()
            .into_iter()
            .filter(|p| p.name().contains(name))
            .collect()
    }

    pub fn list_processes_by_user(&mut self, user: &str) -> Vec<&sysinfo::Process> {
        self.list_processes()
            .into_iter()
            .filter(|p| {
                p.user_id()
                    .and_then(|uid| self.system.get_user_by_id(uid))
                    .map_or(false, |u| u.name() == user)
            })
            .collect()
    }

    pub fn kill_process(&mut self, pid: usize) -> bool {
        self.system.refresh_all();
        if let Some(process) = self.system.process(sysinfo::Pid::from(pid as i32)) {
            process.kill(sysinfo::Signal::Kill)
        } else {
            false
        }
    }

    pub fn restart_process(&mut self, _pid: usize) -> bool {
        false // Placeholder: no restart logic yet
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
        "History feature is not yet implemented.".to_string()
    }
}
y
