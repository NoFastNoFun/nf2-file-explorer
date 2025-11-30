use std::path::PathBuf;
use std::process::Command;

pub fn open_terminal(path: &PathBuf) -> Result<(), String> {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("powershell");
        cmd.arg("-NoExit");
        cmd.arg("-Command");
        cmd.arg(format!("cd '{}'", path.to_string_lossy()));
        cmd.current_dir(path);
        cmd.spawn().map_err(|e| format!("Failed to open terminal: {}", e))?;
    }
    
    #[cfg(not(windows))]
    {
        let mut cmd = Command::new("x-terminal-emulator");
        cmd.arg("--working-directory");
        cmd.arg(path);
        cmd.spawn().map_err(|e| format!("Failed to open terminal: {}", e))?;
    }
    
    Ok(())
}

pub fn run_command_in_terminal(path: &PathBuf, command: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("powershell");
        cmd.arg("-NoExit");
        cmd.arg("-Command");
        cmd.arg(format!("cd '{}'; {}", path.to_string_lossy(), command));
        cmd.current_dir(path);
        cmd.spawn().map_err(|e| format!("Failed to run command: {}", e))?;
    }
    
    #[cfg(not(windows))]
    {
        let mut cmd = Command::new("x-terminal-emulator");
        cmd.arg("--working-directory");
        cmd.arg(path);
        cmd.arg("-e");
        cmd.arg("bash");
        cmd.arg("-c");
        cmd.arg(format!("cd '{}' && {}", path.to_string_lossy(), command));
        cmd.spawn().map_err(|e| format!("Failed to run command: {}", e))?;
    }
    
    Ok(())
}


