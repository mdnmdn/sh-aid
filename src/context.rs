use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command;
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    pub os_type: String,
    pub os_release: String,
    pub platform: String,
    pub arch: String,
    pub shell: String,
    pub current_dir: String,
    pub home_dir: String,
    pub cpu_model: String,
    pub cpu_cores: usize,
    pub total_memory_mb: u64,
    pub free_memory_mb: u64,
    pub directory_listing: String,
}

impl SystemContext {
    pub fn gather() -> Result<Self> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let os_type = env::consts::OS.to_string();
        let platform = env::consts::FAMILY.to_string();
        let arch = env::consts::ARCH.to_string();

        // Get OS release/version information
        let os_release = get_os_release().unwrap_or_else(|| "unknown".to_string());

        let shell = env::var("SHELL").unwrap_or_else(|_| "unknown".to_string());

        let current_dir = env::current_dir()
            .context("Failed to get current directory")?
            .to_string_lossy()
            .to_string();

        let home_dir = dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Get CPU information
        let cpus = sys.cpus();
        let cpu_model = cpus
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let cpu_cores = cpus.len();

        // Get memory information (convert from bytes to MB)
        let total_memory_mb = sys.total_memory() / 1024 / 1024;
        let free_memory_mb = sys.available_memory() / 1024 / 1024;

        // Get directory listing
        let directory_listing = get_directory_listing()
            .unwrap_or_else(|e| format!("Unable to get directory listing: {e}"));

        Ok(SystemContext {
            os_type,
            os_release,
            platform,
            arch,
            shell,
            current_dir,
            home_dir,
            cpu_model,
            cpu_cores,
            total_memory_mb,
            free_memory_mb,
            directory_listing,
        })
    }

    pub fn build_environment_context(&self) -> String {
        format!(
            r#"
Operating System: {} {} ({} - {})
Shell: {}
Current Working Directory: {}
Home Directory: {}
CPU Info: {} ({} cores)
Total Memory: {} MB
Free Memory: {} MB
"#,
            self.os_type,
            self.os_release,
            self.platform,
            self.arch,
            self.shell,
            self.current_dir,
            self.home_dir,
            self.cpu_model,
            self.cpu_cores,
            self.total_memory_mb,
            self.free_memory_mb
        )
    }

    pub fn build_full_context(&self) -> String {
        format!(
            "{}
Result of `ls -l` in working directory:
{}",
            self.build_environment_context(),
            self.directory_listing
        )
    }
}

fn get_os_release() -> Option<String> {
    // Try different methods to get OS version information
    if cfg!(target_os = "macos") {
        get_macos_version()
    } else if cfg!(target_os = "linux") {
        get_linux_version()
    } else if cfg!(target_os = "windows") {
        get_windows_version()
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
fn get_macos_version() -> Option<String> {
    let output = Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()?;

    String::from_utf8(output.stdout)
        .ok()
        .map(|s| s.trim().to_string())
}

#[cfg(not(target_os = "macos"))]
fn get_macos_version() -> Option<String> {
    None
}

#[cfg(target_os = "linux")]
fn get_linux_version() -> Option<String> {
    // Try to read from /etc/os-release first
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return Some(line.split('=').nth(1)?.trim_matches('"').to_string());
            }
        }
    }

    // Fallback to uname
    let output = Command::new("uname").arg("-r").output().ok()?;

    String::from_utf8(output.stdout)
        .ok()
        .map(|s| s.trim().to_string())
}

#[cfg(not(target_os = "linux"))]
fn get_linux_version() -> Option<String> {
    None
}

#[cfg(target_os = "windows")]
fn get_windows_version() -> Option<String> {
    let output = Command::new("cmd").args(&["/C", "ver"]).output().ok()?;

    String::from_utf8(output.stdout)
        .ok()
        .map(|s| s.trim().to_string())
}

#[cfg(not(target_os = "windows"))]
fn get_windows_version() -> Option<String> {
    None
}

fn get_directory_listing() -> Result<String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "dir"])
            .output()
            .context("Failed to execute 'dir' command")?
    } else {
        Command::new("ls")
            .output()
            .context("Failed to execute 'ls' command")?
    };

    if !output.status.success() {
        anyhow::bail!(
            "Directory listing command failed with exit code: {}",
            output.status
        );
    }

    String::from_utf8(output.stdout).context("Failed to convert directory listing output to UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_context_creation() {
        let context = SystemContext::gather();
        assert!(context.is_ok());

        let ctx = context.unwrap();
        assert!(!ctx.os_type.is_empty());
        assert!(!ctx.platform.is_empty());
        assert!(!ctx.arch.is_empty());
        assert!(!ctx.current_dir.is_empty());
        assert!(ctx.cpu_cores > 0);
        assert!(ctx.total_memory_mb > 0);
    }

    #[test]
    fn test_environment_context_format() {
        let context = SystemContext {
            os_type: "linux".to_string(),
            os_release: "20.04".to_string(),
            platform: "unix".to_string(),
            arch: "x86_64".to_string(),
            shell: "/bin/bash".to_string(),
            current_dir: "/home/user".to_string(),
            home_dir: "/home/user".to_string(),
            cpu_model: "Intel Core i7".to_string(),
            cpu_cores: 8,
            total_memory_mb: 16384,
            free_memory_mb: 8192,
            directory_listing: "file1\nfile2".to_string(),
        };

        let env_context = context.build_environment_context();
        assert!(env_context.contains("Operating System: linux 20.04"));
        assert!(env_context.contains("Shell: /bin/bash"));
        assert!(env_context.contains("CPU Info: Intel Core i7 (8 cores)"));
        assert!(env_context.contains("Total Memory: 16384 MB"));
    }

    #[test]
    fn test_full_context_includes_directory_listing() {
        let context = SystemContext {
            os_type: "linux".to_string(),
            os_release: "20.04".to_string(),
            platform: "unix".to_string(),
            arch: "x86_64".to_string(),
            shell: "/bin/bash".to_string(),
            current_dir: "/home/user".to_string(),
            home_dir: "/home/user".to_string(),
            cpu_model: "Intel Core i7".to_string(),
            cpu_cores: 8,
            total_memory_mb: 16384,
            free_memory_mb: 8192,
            directory_listing: "file1\nfile2".to_string(),
        };

        let full_context = context.build_full_context();
        assert!(full_context.contains("Result of `ls -l` in working directory:"));
        assert!(full_context.contains("file1\nfile2"));
    }

    #[test]
    fn test_directory_listing_fallback() {
        // This test verifies that directory listing returns a meaningful error message
        // when the command fails, rather than panicking
        let result = get_directory_listing();
        // The result should either be Ok or contain an error message
        match result {
            Ok(listing) => assert!(!listing.is_empty()),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }
}
