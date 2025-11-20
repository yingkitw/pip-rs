/// Shell completion generation for bash, zsh, fish, and powershell
use crate::errors::PipError;

pub async fn handle_completion(shell: String) -> Result<i32, PipError> {
    match shell.to_lowercase().as_str() {
        "bash" => {
            print_bash_completion();
            Ok(0)
        }
        "zsh" => {
            print_zsh_completion();
            Ok(0)
        }
        "fish" => {
            print_fish_completion();
            Ok(0)
        }
        "powershell" | "pwsh" => {
            print_powershell_completion();
            Ok(0)
        }
        _ => {
            eprintln!("ERROR: Unknown shell: {}. Supported: bash, zsh, fish, powershell", shell);
            Ok(1)
        }
    }
}

fn print_bash_completion() {
    println!(
        r#"# bash completion for pip
# To install: pip completion bash | sudo tee /etc/bash_completion.d/pip

_pip_completion() {{
    local cur prev opts
    COMPREPLY=()
    cur="${{COMP_WORDS[COMP_CWORD]}}"
    prev="${{COMP_WORDS[COMP_CWORD-1]}}"
    
    opts="install uninstall list show search check update freeze download lock debug help"
    
    case "${{prev}}" in
        pip)
            COMPREPLY=( $(compgen -W "${{opts}}" -- ${{cur}}) )
            return 0
            ;;
        install|download)
            COMPREPLY=( $(compgen -W "-r --requirements -o --output -t --target" -- ${{cur}}) )
            return 0
            ;;
        uninstall)
            COMPREPLY=( $(compgen -W "-y --yes" -- ${{cur}}) )
            return 0
            ;;
        list)
            COMPREPLY=( $(compgen -W "--outdated" -- ${{cur}}) )
            return 0
            ;;
        check)
            COMPREPLY=( $(compgen -W "-p --package" -- ${{cur}}) )
            return 0
            ;;
        freeze)
            COMPREPLY=( $(compgen -W "-o --output" -- ${{cur}}) )
            return 0
            ;;
        lock)
            COMPREPLY=( $(compgen -W "-r --requirements -o --output" -- ${{cur}}) )
            return 0
            ;;
        *)
            COMPREPLY=( $(compgen -W "${{opts}}" -- ${{cur}}) )
            ;;
    esac
    
    return 0
}}

complete -o bashdefault -o default -o nospace -F _pip_completion pip
"#
    );
}

fn print_zsh_completion() {
    println!(
        r#"# zsh completion for pip
# To install: pip completion zsh | sudo tee /usr/share/zsh/site-functions/_pip

#compdef pip

_pip_commands() {{
    local -a commands
    commands=(
        'install:Install packages'
        'uninstall:Uninstall packages'
        'list:List installed packages'
        'show:Show package information'
        'search:Search for packages'
        'check:Check for outdated packages'
        'update:Update packages'
        'freeze:Generate requirements.txt'
        'download:Download packages'
        'lock:Generate lock file'
        'debug:Display debug information'
        'help:Print help'
    )
    _describe 'command' commands
}}

_pip_options() {{
    case "${{words[2]}}" in
        install|download)
            _arguments \
                '-r[Requirements file]' \
                '--requirements[Requirements file]' \
                '-o[Output directory]' \
                '--output[Output directory]' \
                '-t[Target directory]' \
                '--target[Target directory]'
            ;;
        uninstall)
            _arguments \
                '-y[Assume yes]' \
                '--yes[Assume yes]'
            ;;
        list)
            _arguments \
                '--outdated[Show outdated packages]'
            ;;
        check)
            _arguments \
                '-p[Package name]' \
                '--package[Package name]'
            ;;
        freeze)
            _arguments \
                '-o[Output file]' \
                '--output[Output file]'
            ;;
        lock)
            _arguments \
                '-r[Requirements file]' \
                '--requirements[Requirements file]' \
                '-o[Output file]' \
                '--output[Output file]'
            ;;
    esac
}}

_pip() {{
    if (( CURRENT == 2 )); then
        _pip_commands
    else
        _pip_options
    fi
}}

_pip
"#
    );
}

fn print_fish_completion() {
    println!(
        r#"# fish completion for pip
# To install: pip completion fish | sudo tee /usr/share/fish/vendor_completions.d/pip.fish

complete -c pip -f -n "__fish_use_subcommand_from_list install uninstall list show search check update freeze download lock debug" -d "Subcommand"

# Commands
complete -c pip -n "__fish_use_subcommand_from_list" -a install -d "Install packages"
complete -c pip -n "__fish_use_subcommand_from_list" -a uninstall -d "Uninstall packages"
complete -c pip -n "__fish_use_subcommand_from_list" -a list -d "List installed packages"
complete -c pip -n "__fish_use_subcommand_from_list" -a show -d "Show package information"
complete -c pip -n "__fish_use_subcommand_from_list" -a search -d "Search for packages"
complete -c pip -n "__fish_use_subcommand_from_list" -a check -d "Check for outdated packages"
complete -c pip -n "__fish_use_subcommand_from_list" -a update -d "Update packages"
complete -c pip -n "__fish_use_subcommand_from_list" -a freeze -d "Generate requirements.txt"
complete -c pip -n "__fish_use_subcommand_from_list" -a download -d "Download packages"
complete -c pip -n "__fish_use_subcommand_from_list" -a lock -d "Generate lock file"
complete -c pip -n "__fish_use_subcommand_from_list" -a debug -d "Display debug information"

# Options for install
complete -c pip -n "__fish_seen_subcommand_from install" -s r -l requirements -d "Requirements file"
complete -c pip -n "__fish_seen_subcommand_from install" -s o -l output -d "Output directory"
complete -c pip -n "__fish_seen_subcommand_from install" -s t -l target -d "Target directory"

# Options for uninstall
complete -c pip -n "__fish_seen_subcommand_from uninstall" -s y -l yes -d "Assume yes"

# Options for list
complete -c pip -n "__fish_seen_subcommand_from list" -l outdated -d "Show outdated packages"

# Options for check
complete -c pip -n "__fish_seen_subcommand_from check" -s p -l package -d "Package name"

# Options for freeze
complete -c pip -n "__fish_seen_subcommand_from freeze" -s o -l output -d "Output file"

# Options for lock
complete -c pip -n "__fish_seen_subcommand_from lock" -s r -l requirements -d "Requirements file"
complete -c pip -n "__fish_seen_subcommand_from lock" -s o -l output -d "Output file"

# Global options
complete -c pip -s h -l help -d "Print help"
complete -c pip -s V -l version -d "Print version"
"#
    );
}

fn print_powershell_completion() {
    println!(
        r#"# PowerShell completion for pip
# To install: pip completion powershell | Out-File -Encoding utf8 $PROFILE\..\pip_completion.ps1
# Then add to your profile: . $PROFILE\..\pip_completion.ps1

$scriptblock = {{
    param($wordToComplete, $commandAst, $cursorPosition)
    
    $command = $commandAst.CommandElements[0].Value
    $words = $commandAst.CommandElements | ForEach-Object {{ $_.Value }}
    
    $commands = @('install', 'uninstall', 'list', 'show', 'search', 'check', 'update', 'freeze', 'download', 'lock', 'debug', 'help')
    
    if ($words.Count -eq 1) {{
        $commands | Where-Object {{ $_ -like "$wordToComplete*" }} | ForEach-Object {{
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }}
    }}
    else {{
        $subcommand = $words[1]
        
        $options = switch ($subcommand) {{
            'install' {{ @('-r', '--requirements', '-o', '--output', '-t', '--target') }}
            'uninstall' {{ @('-y', '--yes') }}
            'list' {{ @('--outdated') }}
            'check' {{ @('-p', '--package') }}
            'freeze' {{ @('-o', '--output') }}
            'lock' {{ @('-r', '--requirements', '-o', '--output') }}
            default {{ @() }}
        }}
        
        $options | Where-Object {{ $_ -like "$wordToComplete*" }} | ForEach-Object {{
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }}
    }}
}}

Register-ArgumentCompleter -CommandName pip -ScriptBlock $scriptblock
"#
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bash_completion() {
        let result = handle_completion("bash".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_zsh_completion() {
        let result = handle_completion("zsh".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_fish_completion() {
        let result = handle_completion("fish".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_powershell_completion() {
        let result = handle_completion("powershell".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_invalid_shell() {
        let result = handle_completion("invalid".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
}
