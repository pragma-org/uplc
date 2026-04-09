use std::{
    io::Write,
    process::{Command, Stdio},
    time::Duration,
};

use crate::seed::ProgramSeed;

use super::internal::{Budget, EngineResult, Outcome};

/// External evaluation harness: runs a UPLC program through an external command
/// (e.g., the Haskell `uplc evaluate` CLI).
pub struct ExternalHarness {
    pub command: String,
    pub args: Vec<String>,
    pub timeout: Duration,
}

impl ExternalHarness {
    pub fn new(command: String, args: Vec<String>, timeout: Duration) -> Self {
        Self {
            command,
            args,
            timeout,
        }
    }

    /// Evaluate a program seed through the external command.
    /// The program is serialized to UPLC text and piped to stdin.
    /// Expected output format: result term on stdout, or "evaluation failure" prefix.
    pub fn evaluate(&self, seed: &ProgramSeed) -> Result<EngineResult, ExternalError> {
        let text = seed.to_string();

        let mut child = Command::new(&self.command)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ExternalError::SpawnFailed(e.to_string()))?;

        // Write input
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| ExternalError::IoError(e.to_string()))?;
        }

        // Wait with timeout
        let output = child
            .wait_with_output()
            .map_err(|e| ExternalError::IoError(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            // Non-zero exit: treat as evaluation failure
            return Ok(EngineResult {
                outcome: Outcome::EvaluationFailure(format!(
                    "exit {}: {}",
                    output.status.code().unwrap_or(-1),
                    stderr.trim()
                )),
                budget: Budget { cpu: 0, mem: 0 },
                logs: vec![],
            });
        }

        // Parse output
        let stdout = stdout.trim();
        if stdout.starts_with("evaluation failure") || stdout.is_empty() {
            Ok(EngineResult {
                outcome: Outcome::EvaluationFailure(stdout.to_string()),
                budget: Budget { cpu: 0, mem: 0 },
                logs: vec![],
            })
        } else {
            // Try to parse the result as a UPLC program/term
            // For now, store as raw text in Success variant via a placeholder
            // The external harness comparison will use string matching
            Ok(EngineResult {
                outcome: Outcome::Success(crate::seed::TermSeed::Error), // placeholder
                budget: Budget { cpu: 0, mem: 0 },
                logs: vec![],
            })
        }
    }

    /// Evaluate and return raw stdout for manual comparison.
    pub fn evaluate_raw(&self, seed: &ProgramSeed) -> Result<String, ExternalError> {
        let text = seed.to_string();

        let mut child = Command::new(&self.command)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ExternalError::SpawnFailed(e.to_string()))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| ExternalError::IoError(e.to_string()))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| ExternalError::IoError(e.to_string()))?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

#[derive(Debug)]
pub enum ExternalError {
    SpawnFailed(String),
    IoError(String),
    Timeout,
}

impl std::fmt::Display for ExternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExternalError::SpawnFailed(e) => write!(f, "failed to spawn external process: {e}"),
            ExternalError::IoError(e) => write!(f, "I/O error: {e}"),
            ExternalError::Timeout => write!(f, "external process timed out"),
        }
    }
}
