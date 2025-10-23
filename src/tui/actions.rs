use anyhow::Result;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::process::Command;

/// Field-specific actions that can be triggered by keybinds
#[derive(Debug, Clone)]
pub enum FieldAction {
    /// Open external editor for the field value
    ExternalEditor { editor: String, extension: String },
    /// Custom shell command
    CustomCommand { command: String },
}

impl FieldAction {
    /// Execute the action for a given value, returning the new value if changed
    pub fn execute(&self, current_value: &str) -> Result<Option<String>> {
        match self {
            FieldAction::ExternalEditor { editor, extension } => {
                execute_external_editor(current_value, editor, extension)
            }
            FieldAction::CustomCommand { command } => {
                execute_custom_command(command, current_value)
            }
        }
    }
}

fn execute_external_editor(content: &str, editor: &str, extension: &str) -> Result<Option<String>> {
    // Create temp file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("schema-tui-edit.{}", extension));
    std::fs::write(&temp_file, content)?;
    
    // Disable raw mode for editor
    disable_raw_mode()?;
    
    // Launch editor
    let status = Command::new(editor)
        .arg(&temp_file)
        .status();
    
    // Re-enable raw mode
    enable_raw_mode()?;
    
    match status {
        Ok(exit_status) if exit_status.success() => {
            // Read modified content
            let new_content = std::fs::read_to_string(&temp_file)?;
            std::fs::remove_file(&temp_file).ok();
            
            if new_content != content {
                Ok(Some(new_content))
            } else {
                Ok(None)
            }
        }
        Ok(_) => {
            std::fs::remove_file(&temp_file).ok();
            Ok(None)
        }
        Err(e) => {
            std::fs::remove_file(&temp_file).ok();
            Err(e.into())
        }
    }
}

fn execute_custom_command(command: &str, current_value: &str) -> Result<Option<String>> {
    // Set current value as environment variable
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .env("CURRENT_VALUE", current_value)
        .output()?;
    
    if output.status.success() {
        let new_value = String::from_utf8(output.stdout)?.trim().to_string();
        if new_value != current_value && !new_value.is_empty() {
            Ok(Some(new_value))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}
