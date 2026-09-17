//! Native, zero-dependency shell completion generator for Bash, Zsh, and Fish.

use crate::common::error::Result;
use clap::ValueEnum;

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupportedShell {
    /// Bourne Again SHell (bash)
    Bash,
    /// Z Shell (zsh)
    Zsh,
    /// Friendly Interactive SHell (fish)
    Fish,
}

/// Generates shell completion scripts and writes to stdout.
pub fn generate_completions(shell: SupportedShell) -> Result<()> {
    match shell {
        SupportedShell::Bash => print!("{}", generate_bash_completions()),
        SupportedShell::Zsh => print!("{}", generate_zsh_completions()),
        SupportedShell::Fish => print!("{}", generate_fish_completions()),
    }
    Ok(())
}

fn generate_bash_completions() -> &'static str {
    r#"_tidy() {
    local cur prev words cword
    _init_completion || return

    local commands="run watch undo status service init completions"
    local common_opts="--verbose -v --config -c --help -h"

    case "${cword}" in
        1)
            COMPREPLY=( $(compgen -W "${commands} ${common_opts}" -- "${cur}") )
            return 0
            ;;
        2)
            case "${words[1]}" in
                run)
                    COMPREPLY=( $(compgen -W "--path -p --dry-run --recursive -r ${common_opts}" -- "${cur}") )
                    return 0
                    ;;
                watch)
                    COMPREPLY=( $(compgen -W "--path -p --recursive -r --debounce --initial-scan ${common_opts}" -- "${cur}") )
                    return 0
                    ;;
                undo)
                    COMPREPLY=( $(compgen -W "--run-id --last -n --all --dry-run ${common_opts}" -- "${cur}") )
                    return 0
                    ;;
                service)
                    COMPREPLY=( $(compgen -W "enable disable status restart logs --path -p --recursive -r --lines -n --follow -f ${common_opts}" -- "${cur}") )
                    return 0
                    ;;
                init)
                    COMPREPLY=( $(compgen -W "--force -f --path -p ${common_opts}" -- "${cur}") )
                    return 0
                    ;;
                completions)
                    COMPREPLY=( $(compgen -W "bash zsh fish" -- "${cur}") )
                    return 0
                    ;;
            esac
            ;;
    esac

    case "${prev}" in
        --path|-p)
            COMPREPLY=( $(compgen -d -- "${cur}") )
            return 0
            ;;
        --config|-c)
            COMPREPLY=( $(compgen -f -- "${cur}") )
            return 0
            ;;
    esac
}

complete -F _tidy tidy
"#
}

fn generate_zsh_completions() -> &'static str {
    r#"#compdef tidy

_tidy() {
    local -a commands
    commands=(
        'run:Execute a single-pass scan and organize cycle'
        'watch:Start the long-running resident filesystem watcher daemon'
        'undo:Roll back the most recent operation batch using the local ledger'
        'status:Report watched directories, rule summaries, and recent activity'
        'service:Manage user-level OS background daemons (systemd on Linux, launchd on macOS)'
        'init:Scaffold a default configuration file (config.toml)'
        'completions:Generate shell completions for Bash, Zsh, or Fish'
    )

    _arguments -C \
        '(-v --verbose)'{-v,--verbose}'[Verbose output logging]' \
        '(-c --config)'{-c,--config}'[Path to custom configuration file]:file:_files' \
        '(-h --help)'{-h,--help}'[Print help]' \
        '1: :->cmd' \
        '*:: :->args'

    case "$state" in
        cmd)
            _describe -t commands 'tidy command' commands
            ;;
        args)
            case "$words[1]" in
                run)
                    _arguments \
                        '(-p --path)'{-p,--path}'[Target directory to organize]:dir:_files -/' \
                        '--dry-run[Preview organization without modifying disk]' \
                        '(-r --recursive)'{-r,--recursive}'[Scan directory recursively]'
                    ;;
                watch)
                    _arguments \
                        '(-p --path)'{-p,--path}'[Target directory to watch]:dir:_files -/' \
                        '(-r --recursive)'{-r,--recursive}'[Watch directory recursively]' \
                        '--debounce[Debounce duration in ms]:ms:' \
                        '--initial-scan[Perform initial scan on startup]'
                    ;;
                undo)
                    _arguments \
                        '--run-id[Specific Run UUID to revert]:uuid:' \
                        '(-n --last)'{-n,--last}'[Number of runs to revert]:count:' \
                        '--all[Roll back all recorded runs]' \
                        '--dry-run[Preview undo without modifying disk]'
                    ;;
                service)
                    _arguments \
                        '1:action:(enable disable status restart logs)' \
                        '(-p --path)'{-p,--path}'[Target watch directory]:dir:_files -/' \
                        '(-r --recursive)'{-r,--recursive}'[Watch directory recursively]' \
                        '(-n --lines)'{-n,--lines}'[Number of log lines to show]:count:' \
                        '(-f --follow)'{-f,--follow}'[Follow real-time log output]'
                    ;;
                init)
                    _arguments \
                        '(-f --force)'{-f,--force}'[Overwrite existing config]' \
                        '(-p --path)'{-p,--path}'[Custom config destination]:file:_files'
                    ;;
                completions)
                    _arguments '1:shell:(bash zsh fish)'
                    ;;
            esac
            ;;
    esac
}

_tidy "$@"
"#
}

fn generate_fish_completions() -> &'static str {
    r#"# Fish completions for tidy

complete -c tidy -f

# Subcommands
complete -c tidy -n "__fish_use_subcommand" -a "run" -d "Execute a single-pass scan and organize cycle"
complete -c tidy -n "__fish_use_subcommand" -a "watch" -d "Start the resident filesystem watcher daemon"
complete -c tidy -n "__fish_use_subcommand" -a "undo" -d "Roll back the most recent operation batch"
complete -c tidy -n "__fish_use_subcommand" -a "status" -d "Report watched directories, rules, and activity"
complete -c tidy -n "__fish_use_subcommand" -a "service" -d "Manage user-level background services"
complete -c tidy -n "__fish_use_subcommand" -a "init" -d "Scaffold a default configuration file"
complete -c tidy -n "__fish_use_subcommand" -a "completions" -d "Generate shell completion scripts"

# Global options
complete -c tidy -s v -l verbose -d "Verbose output logging"
complete -c tidy -s c -l config -d "Path to custom configuration file" -r
complete -c tidy -s h -l help -d "Print help"

# run options
complete -c tidy -n "__fish_seen_subcommand_from run" -s p -l path -d "Target directory to organize" -a "(__fish_complete_directories)"
complete -c tidy -n "__fish_seen_subcommand_from run" -l dry-run -d "Perform dry run without moving files"
complete -c tidy -n "__fish_seen_subcommand_from run" -s r -l recursive -d "Scan target directory recursively"

# watch options
complete -c tidy -n "__fish_seen_subcommand_from watch" -s p -l path -d "Directory to watch" -a "(__fish_complete_directories)"
complete -c tidy -n "__fish_seen_subcommand_from watch" -s r -l recursive -d "Watch directory recursively"
complete -c tidy -n "__fish_seen_subcommand_from watch" -l debounce -d "Debounce duration in ms"
complete -c tidy -n "__fish_seen_subcommand_from watch" -l initial-scan -d "Perform initial scan on startup"

# undo options
complete -c tidy -n "__fish_seen_subcommand_from undo" -l run-id -d "Specific Run UUID to roll back"
complete -c tidy -n "__fish_seen_subcommand_from undo" -s n -l last -d "Number of runs to roll back"
complete -c tidy -n "__fish_seen_subcommand_from undo" -l all -d "Roll back all recorded runs"
complete -c tidy -n "__fish_seen_subcommand_from undo" -l dry-run -d "Preview undo without modifying disk"

# service options
complete -c tidy -n "__fish_seen_subcommand_from service" -a "enable disable status restart logs"
complete -c tidy -n "__fish_seen_subcommand_from service" -s p -l path -d "Target directory to watch"
complete -c tidy -n "__fish_seen_subcommand_from service" -s r -l recursive -d "Watch directory recursively"
complete -c tidy -n "__fish_seen_subcommand_from service" -s n -l lines -d "Number of log lines to show"
complete -c tidy -n "__fish_seen_subcommand_from service" -s f -l follow -d "Follow real-time log output"

# init options
complete -c tidy -n "__fish_seen_subcommand_from init" -s f -l force -d "Overwrite existing config"
complete -c tidy -n "__fish_seen_subcommand_from init" -s p -l path -d "Custom config path"

# completions options
complete -c tidy -n "__fish_seen_subcommand_from completions" -a "bash zsh fish"
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_bash_completions() {
        let script = generate_bash_completions();
        assert!(script.contains("complete -F _tidy tidy"));
        assert!(script.contains("commands=\"run watch undo status service init completions\""));
    }

    #[test]
    fn test_generate_zsh_completions() {
        let script = generate_zsh_completions();
        assert!(script.contains("#compdef tidy"));
        assert!(script.contains("tidy command"));
    }

    #[test]
    fn test_generate_fish_completions() {
        let script = generate_fish_completions();
        assert!(script.contains("complete -c tidy"));
        assert!(script.contains("__fish_use_subcommand"));
    }
}
