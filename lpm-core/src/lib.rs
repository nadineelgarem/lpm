// src/lib.rs for lpm-core

use sysinfo::{System, SystemExt, ProcessExt, Pid, PidExt};
use chrono::Local;
use libc::{setpriority, PRIO_PROCESS};

pub struct ProcessManager {
    system: System,
    history: Vec<String>,
}

impl ProcessManager {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self { system, history: vec![] }
    }

    pub fn list_processes(&mut self) -> Vec<&sysinfo::Process> {
        self.system.refresh_processes();
        self.system.processes().values().collect()
    }

    pub fn list_processes_by_name(&mut self, name: &str) -> Vec<&sysinfo::Process> {
        self.list_processes()
            .into_iter()
            .filter(|p| p.name().to_lowercase().contains(&name.to_lowercase()))
            .collect()
    }

    pub fn list_processes_by_user(&mut self, user: &str) -> Vec<&sysinfo::Process> {
        self.list_processes()
            .into_iter()
            .filter(|p| p.user_id().map_or(false, |u| u.to_string() == user))
            .collect()
    }

    pub fn kill_process(&mut self, pid: usize) -> bool {
        let pid = Pid::from(pid);
        if let Some(p) = self.system.process(pid) {
            let result = p.kill();
            self.history.push(format!("{} KILLED PID {}", Local::now(), pid));
            result
        } else {
            false
        }
    }

    pub fn change_priority(&mut self, pid: usize, new_nice: i32) -> bool {
        let result = unsafe { setpriority(PRIO_PROCESS, pid as u32, new_nice) == 0 };
        self.history.push(format!("{} CHANGED PRIORITY PID {} TO {}", Local::now(), pid, new_nice));
        result
    }

    pub fn get_process_tree(&mut self) -> Vec<(usize, String, Option<usize>)> {
        self.system.refresh_processes();
        self.system
            .processes()
            .values()
            .map(|p| (
                p.pid().as_u32() as usize,
                p.name().to_string(),
                p.parent().map(|pp| pp.as_u32() as usize),
            ))
            .collect()
    }

    pub fn restart_process(&mut self, pid: usize) -> bool {
        if let Some(p) = self.system.process(Pid::from(pid)) {
            let cmd = p.cmd().join(" ");
            let _ = std::process::Command::new("kill").arg("-9").arg(pid.to_string()).status();
            self.history.push(format!("{} RESTARTED PID {}", Local::now(), pid));
            std::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .spawn()
                .is_ok()
        } else {
            false
        }
    }

    pub fn export_processes(&mut self, _format: &str, file_path: &str) -> Result<(), String> {
        let data = self.list_processes()
            .into_iter()
            .map(|p| format!("[{}] {} ({}% CPU, {} KB Memory, USER: {:?})", p.pid(), p.name(), p.cpu_usage(), p.memory(), p.user_id()))
            .collect::<Vec<String>>()
            .join("\n");
        std::fs::write(file_path, data).map_err(|e| e.to_string())
    }

    pub fn check_alerts(&mut self, cpu_threshold: f32, mem_threshold: u64) -> Vec<String> {
        self.list_processes()
            .into_iter()
            .filter(|p| p.cpu_usage() > cpu_threshold || p.memory() > mem_threshold)
            .map(|p| format!(
                "ALERT: [{}] {} CPU: {}% MEM: {} KB",
                p.pid(), p.name(), p.cpu_usage(), p.memory()
            ))
            .collect()
    }

    pub fn show_history(&self) -> String {
        if self.history.is_empty() {
            "No history yet.".to_string()
        } else {
            self.history.join("\n")
        }
    }
}

