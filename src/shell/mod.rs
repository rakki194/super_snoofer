pub mod aliases;
pub mod integration;

pub use integration::{
    install_shell_integration,
    uninstall_shell_integration,
    add_to_shell_config,
    detect_shell_config,
};
