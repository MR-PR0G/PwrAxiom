use std::io::Write;
use std::process::Command;

pub fn execute_batch(password: &str, commands: &[String]) -> Result<(), String> {
    if commands.is_empty() {
        return Ok(());
    }
    let batch_script = commands.join("\n");
    let mut child = Command::new("sudo")
        .arg("-S")
        .arg("sh")
        .arg("-c")
        .arg(&batch_script)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn sudo: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = writeln!(stdin, "{}", password);
    }

    let output = child.wait_with_output().map_err(|e| format!("Wait failed: {}", e))?;
    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Execution failed: {}", err_msg));
    }
    Ok(())
}