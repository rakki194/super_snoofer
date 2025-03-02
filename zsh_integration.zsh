# Super Snoofer ZSH Integration
# This script provides advanced command correction, learning, and auto-completion
# Add to your .zshrc: source /path/to/zsh_integration.zsh

# Configuration (customize as needed)
SUPER_SNOOFER_CMD="super_snoofer"
SUPER_SNOOFER_ENABLED=true
SUPER_SNOOFER_COMPLETION_ENABLED=true
SUPER_SNOOFER_SUGGESTIONS_ENABLED=true  # Enable/disable real-time suggestions
SUPER_SNOOFER_EARLY_SUGGESTIONS=true    # Enable/disable suggestions after first keypress
SUPER_SNOOFER_FULL_COMPLETIONS=true     # Prioritize full command completions over partial ones
SUPER_SNOOFER_TYPO_CORRECTION=true      # Prioritize typo correction when using Tab

# Commands to exclude from auto-correction (space-separated)
SUPER_SNOOFER_EXCLUDE_COMMANDS="vim vi nano emacs cd ls cat grep find curl wget"

# Flag to prevent recursive correction
__super_snoofer_running=false

# Store the current suggestion
__super_snoofer_suggestion=""
__super_snoofer_suggestion_displayed=false
__super_snoofer_correction=""

# Helper to check if a command should be excluded
__super_snoofer_should_exclude() {
  local cmd="$1"
  if [[ -z "$cmd" ]]; then
    return 1
  fi
  for exclude in $SUPER_SNOOFER_EXCLUDE_COMMANDS; do
    if [[ "$cmd" == "$exclude" ]]; then
      return 0
    fi
  done
  return 1
}

# Function to get the first word (command) from a command line
__super_snoofer_get_command() {
  echo "$1" | awk '{print $1}'
}

# Function to record successful commands for learning
__super_snoofer_record_valid_command() {
  local cmd="$1"
  # Skip recording if disabled or if it's a built-in command
  if [[ "$SUPER_SNOOFER_ENABLED" != "true" ]]; then
    return
  fi
  
  # Don't record empty commands or excluded commands
  if [[ -z "$cmd" ]]; then
    return
  fi
  
  local first_cmd="$(__super_snoofer_get_command "$cmd")"
  if __super_snoofer_should_exclude "$first_cmd"; then
    return
  fi
  
  # Record the valid command in the background
  (
    $SUPER_SNOOFER_CMD --record-valid-command "$cmd" &>/dev/null &
  )
}

# Function to check and correct a command before execution
__super_snoofer_check_command() {
  local cmd="$1"
  
  # Skip if empty, disabled, or if Super Snoofer is already running
  if [[ -z "$cmd" ]] || [[ "$SUPER_SNOOFER_ENABLED" != "true" ]] || [[ "$__super_snoofer_running" == "true" ]]; then
    return
  fi
  
  local first_word="$(__super_snoofer_get_command "$cmd")"
  
  # Skip excluded commands or commands starting with space
  if [[ -z "$first_word" ]] || __super_snoofer_should_exclude "$first_word" || [[ "$cmd" == " "* ]]; then
    return
  fi
  
  # Skip if the command is a shell builtin or alias
  if type "$first_word" &>/dev/null; then
    return
  fi
  
  # Set flag to prevent recursive correction
  __super_snoofer_running=true
  
  # Check if the command needs correction
  local corrected=$($SUPER_SNOOFER_CMD --check-command "$cmd" 2>/dev/null)
  local exit_code=$?
  
  # Reset flag
  __super_snoofer_running=false
  
  # If the command was corrected and different from the original
  if [[ $exit_code -eq 0 ]] && [[ "$corrected" != "$cmd" ]]; then
    echo -e "\033[0;33mCommand corrected: \033[0;32m$corrected\033[0m"
    
    # Record the correction
    (
      $SUPER_SNOOFER_CMD --record-correction "$cmd" "$corrected" &>/dev/null &
    )
    
    # Replace the command in the buffer
    BUFFER="$corrected"
    # Move the cursor to the end
    CURSOR=${#BUFFER}
    
    # Return value of 0 means we've handled the command
    return 0
  fi
  
  # Return value of 1 means proceed with the original command
  return 1
}

# Function to check for typos in current command
__super_snoofer_check_typos() {
  local cmd="$1"
  
  # Skip if empty or if Super Snoofer is already running
  if [[ -z "$cmd" ]] || [[ "$__super_snoofer_running" == "true" ]]; then
    return 1
  fi
  
  # Skip excluded commands
  local first_word="$(__super_snoofer_get_command "$cmd")"
  if [[ -z "$first_word" ]] || __super_snoofer_should_exclude "$first_word"; then
    return 1
  fi
  
  # Set flag to prevent recursive correction
  __super_snoofer_running=true
  
  # Check for typo correction
  local corrected=$($SUPER_SNOOFER_CMD --check-command "$cmd" 2>/dev/null)
  local exit_code=$?
  
  # Reset flag
  __super_snoofer_running=false
  
  # If we got a correction
  if [[ $exit_code -eq 0 ]] && [[ -n "$corrected" ]] && [[ "$corrected" != "$cmd" ]]; then
    __super_snoofer_correction="$corrected"
    return 0
  fi
  
  __super_snoofer_correction=""
  return 1
}

# Function to get real-time suggestions as user types
__super_snoofer_suggest() {
  # Only provide suggestions if enabled
  if [[ "$SUPER_SNOOFER_SUGGESTIONS_ENABLED" != "true" ]]; then
    return
  fi
  
  # For early suggestions, we want to provide suggestions even when the buffer has just one character
  # For non-early suggestions, we only provide when cursor is at the end
  if [[ "$SUPER_SNOOFER_EARLY_SUGGESTIONS" != "true" && (-z "$BUFFER" || "$CURSOR" -ne "${#BUFFER}") ]] || 
     [[ "$SUPER_SNOOFER_EARLY_SUGGESTIONS" == "true" && -z "$BUFFER" ]]; then
    # Clear any existing suggestion
    if [[ "$__super_snoofer_suggestion_displayed" == "true" ]]; then
      __super_snoofer_clear_suggestion
    fi
    return
  fi
  
  # Don't suggest if cursor is not at the end (fixed condition)
  if [[ "$CURSOR" -ne "${#BUFFER}" ]]; then
    if [[ "$__super_snoofer_suggestion_displayed" == "true" ]]; then
      __super_snoofer_clear_suggestion
    fi
    return
  fi
  
  # Get the current command
  local cmd="$BUFFER"
  local first_char="${cmd:0:1}"
  local first_word="$(__super_snoofer_get_command "$cmd")"
  
  # Skip if command should be excluded
  if [[ ${#cmd} -gt 1 && -n "$first_word" ]] && __super_snoofer_should_exclude "$first_word"; then
    __super_snoofer_clear_suggestion
    return
  fi
  
  # Reset the correction
  __super_snoofer_correction=""
  
  # For early suggestions with just one character, check if it's a valid first character
  # Only proceed if we're in early suggestions mode or we have more than one character
  if [[ "$SUPER_SNOOFER_EARLY_SUGGESTIONS" == "true" || ${#cmd} -gt 1 ]]; then
    # Get suggestion using super_snoofer
    __super_snoofer_running=true
    
    local suggestion=""
    
    # First, check for typos if enabled
    if [[ "$SUPER_SNOOFER_TYPO_CORRECTION" == "true" && ${#cmd} -gt 2 ]]; then
      # Only check for typos in longer commands to avoid false corrections
      local corrected=$($SUPER_SNOOFER_CMD --check-command "$cmd" 2>/dev/null)
      if [[ -n "$corrected" && "$corrected" != "$cmd" ]]; then
        __super_snoofer_correction="$corrected"
        # Use the correction as a suggestion
        suggestion="$corrected"
      fi
    fi
    
    # Continue with completions if no typo correction or if suggestion option enabled
    if [[ -z "$suggestion" || "$SUPER_SNOOFER_SUGGESTIONS_ENABLED" == "true" ]]; then
      # Use different flag based on whether full completions are enabled
      if [[ "$SUPER_SNOOFER_FULL_COMPLETIONS" == "true" ]]; then
        # Request full command completion (try to complete to the end)
        local full_suggestion=$($SUPER_SNOOFER_CMD --suggest-full-completion "$cmd" 2>/dev/null)
        
        # If no full completion available, fall back to regular completion
        if [[ -n "$full_suggestion" && "$full_suggestion" != "$cmd" ]]; then
          suggestion="$full_suggestion"
        else
          local partial_suggestion=$($SUPER_SNOOFER_CMD --suggest-completion "$cmd" 2>/dev/null)
          if [[ -n "$partial_suggestion" && "$partial_suggestion" != "$cmd" ]]; then
            suggestion="$partial_suggestion"
          fi
        fi
      else
        # Standard completion behavior
        local partial_suggestion=$($SUPER_SNOOFER_CMD --suggest-completion "$cmd" 2>/dev/null)
        if [[ -n "$partial_suggestion" && "$partial_suggestion" != "$cmd" ]]; then
          suggestion="$partial_suggestion"
        fi
      fi
      
      # Additionally, try to get a frequently used complete command (full history match)
      if [[ "$SUPER_SNOOFER_FULL_COMPLETIONS" == "true" ]]; then
        local frequent_cmd=$($SUPER_SNOOFER_CMD --suggest-frequent-command "$cmd" 2>/dev/null)
        
        # If we got a frequent command and it's longer than the current suggestion, use it instead
        if [[ -n "$frequent_cmd" && "$frequent_cmd" != "$cmd" && ${#frequent_cmd} -gt ${#suggestion} ]]; then
          suggestion="$frequent_cmd"
        fi
      fi
    fi
    
    __super_snoofer_running=false
    
    # If we got a suggestion that's different from the current input
    if [[ -n "$suggestion" && "$suggestion" != "$cmd" ]]; then
      # Store the suggestion
      __super_snoofer_suggestion="$suggestion"
      
      # Calculate the part to display as suggestion (only the part after what user typed)
      local suffix="${suggestion:${#cmd}}"
      
      # Display the suggestion in a faded color if we have a suffix
      if [[ -n "$suffix" ]]; then
        # Save cursor position
        local pos=$CURSOR
        
        # Display suggestion
        BUFFER="$cmd$suffix"
        
        # Use a different highlight color for corrections vs completions
        if [[ -n "$__super_snoofer_correction" ]]; then
          # Highlight typo corrections in a more noticeable color (light blue)
          region_highlight=("$pos $((pos + ${#suffix})) fg=81")
        else
          # Normal completion suggestions (gray)
          region_highlight=("$pos $((pos + ${#suffix})) fg=8")
        fi
        
        # Move cursor back to original position
        CURSOR=$pos
        
        __super_snoofer_suggestion_displayed=true
      else
        __super_snoofer_clear_suggestion
      fi
    else
      __super_snoofer_clear_suggestion
    fi
  else
    __super_snoofer_clear_suggestion
  fi
  
  # Force redisplay
  zle -R
}

# Function to clear the current suggestion
__super_snoofer_clear_suggestion() {
  if [[ "$__super_snoofer_suggestion_displayed" == "true" ]]; then
    BUFFER="${BUFFER:0:$CURSOR}"
    region_highlight=()
    __super_snoofer_suggestion=""
    __super_snoofer_suggestion_displayed=false
    zle -R
  fi
}

# Function to accept the current suggestion
__super_snoofer_accept_suggestion() {
  # First check if we have a displayed suggestion
  if [[ "$__super_snoofer_suggestion_displayed" == "true" && -n "$__super_snoofer_suggestion" ]]; then
    BUFFER="$__super_snoofer_suggestion"
    CURSOR=${#BUFFER}
    __super_snoofer_suggestion=""
    __super_snoofer_suggestion_displayed=false
    __super_snoofer_correction=""
    region_highlight=()
    zle -R
    return
  fi
  
  # If no suggestion is displayed, try to check for typos
  if [[ "$SUPER_SNOOFER_TYPO_CORRECTION" == "true" && -n "$BUFFER" ]]; then
    if __super_snoofer_check_typos "$BUFFER"; then
      # Apply the typo correction
      local corrected="$__super_snoofer_correction"
      if [[ -n "$corrected" && "$corrected" != "$BUFFER" ]]; then
        echo -e "\033[0;33mCommand corrected: \033[0;32m$corrected\033[0m"
        BUFFER="$corrected"
        CURSOR=${#BUFFER}
        __super_snoofer_correction=""
        zle -R
        return
      fi
    fi
  fi
  
  # If no suggestion and no typo correction, perform default tab completion
  zle .expand-or-complete
}

# Function for special keys that should clear suggestion
__super_snoofer_special_key() {
  __super_snoofer_clear_suggestion
  zle .$WIDGET
}

# Function to update suggestions whenever the line is redrawn
# This approach works with other plugins and doesn't require overriding self-insert
__super_snoofer_zle_line_pre_redraw() {
  # Only trigger suggestion logic if the buffer has changed since last time
  if [[ "$BUFFER" != "$__super_snoofer_last_buffer" ]]; then
    __super_snoofer_last_buffer="$BUFFER"
    __super_snoofer_suggest
  fi
}

# Set up the preexec hook (runs before command execution)
__super_snoofer_preexec() {
  # Extract the actual command (remove leading spaces and environment variables)
  local cmd=$(echo "$1" | sed -E 's/^ +//; s/^[A-Za-z0-9_]+=([^ ]+ )*//g')
  
  # Check and correct the command
  __super_snoofer_check_command "$cmd"
}

# Set up the precmd hook (runs after command execution)
__super_snoofer_precmd() {
  # Record the last successful command
  if [[ $? -eq 0 ]] && [[ -n "$__super_snoofer_last_cmd" ]]; then
    __super_snoofer_record_valid_command "$__super_snoofer_last_cmd"
  fi
  __super_snoofer_last_cmd=""
  
  # Reset the last buffer to avoid immediate suggestions
  __super_snoofer_last_buffer=""
}

# Function to be called before ZLE accepts a line
__super_snoofer_accept_line() {
  __super_snoofer_last_cmd="$BUFFER"
  zle .accept-line
}

# Track the last buffer to detect changes
__super_snoofer_last_buffer=""

# Add our custom widgets and hooks
zle -N accept-line __super_snoofer_accept_line
zle -N __super_snoofer_accept_suggestion_widget __super_snoofer_accept_suggestion
zle -N zle-line-pre-redraw __super_snoofer_zle_line_pre_redraw

# Special keys that should clear suggestion
for key in backward-delete-char backward-word forward-word beginning-of-line end-of-line kill-line kill-word delete-word; do
  zle -N $key __super_snoofer_special_key
done

# Bind tab to either accept suggestion or do normal completion
bindkey '^I' __super_snoofer_accept_suggestion_widget

# Set up the ZSH hooks if they don't already exist
autoload -Uz add-zsh-hook
add-zsh-hook preexec __super_snoofer_preexec
add-zsh-hook precmd __super_snoofer_precmd

# Enable auto-completion if configured
if [[ "$SUPER_SNOOFER_COMPLETION_ENABLED" == "true" ]]; then
  # Source the auto-completion file if it exists
  if [[ -f ~/.zsh_super_snoofer_completions ]]; then
    source ~/.zsh_super_snoofer_completions
  fi
fi

# Function to toggle Super Snoofer on/off
super_snoofer_toggle() {
  if [[ "$SUPER_SNOOFER_ENABLED" == "true" ]]; then
    SUPER_SNOOFER_ENABLED=false
    echo "Super Snoofer disabled 🐺❌"
  else
    SUPER_SNOOFER_ENABLED=true
    echo "Super Snoofer enabled 🐺✅"
  fi
}

# Function to toggle suggestions on/off
super_snoofer_toggle_suggestions() {
  if [[ "$SUPER_SNOOFER_SUGGESTIONS_ENABLED" == "true" ]]; then
    SUPER_SNOOFER_SUGGESTIONS_ENABLED=false
    echo "Super Snoofer suggestions disabled 🐺❌"
  else
    SUPER_SNOOFER_SUGGESTIONS_ENABLED=true
    echo "Super Snoofer suggestions enabled 🐺✅"
  fi
}

# Function to toggle early suggestions on/off
super_snoofer_toggle_early_suggestions() {
  if [[ "$SUPER_SNOOFER_EARLY_SUGGESTIONS" == "true" ]]; then
    SUPER_SNOOFER_EARLY_SUGGESTIONS=false
    echo "Super Snoofer early suggestions disabled 🐺❌"
  else
    SUPER_SNOOFER_EARLY_SUGGESTIONS=true
    echo "Super Snoofer early suggestions enabled 🐺✅"
  fi
}

# Function to toggle full completions on/off
super_snoofer_toggle_full_completions() {
  if [[ "$SUPER_SNOOFER_FULL_COMPLETIONS" == "true" ]]; then
    SUPER_SNOOFER_FULL_COMPLETIONS=false
    echo "Super Snoofer full completions disabled 🐺❌"
  else
    SUPER_SNOOFER_FULL_COMPLETIONS=true
    echo "Super Snoofer full completions enabled 🐺✅"
  fi
}

# Function to toggle typo correction on/off
super_snoofer_toggle_typo_correction() {
  if [[ "$SUPER_SNOOFER_TYPO_CORRECTION" == "true" ]]; then
    SUPER_SNOOFER_TYPO_CORRECTION=false
    echo "Super Snoofer typo correction disabled 🐺❌"
  else
    SUPER_SNOOFER_TYPO_CORRECTION=true
    echo "Super Snoofer typo correction enabled 🐺✅"
  fi
}

# Function to reload completions
super_snoofer_reload_completions() {
  if [[ "$SUPER_SNOOFER_COMPLETION_ENABLED" == "true" ]]; then
    $SUPER_SNOOFER_CMD --export-completions ~/.zsh_super_snoofer_completions &>/dev/null
    
    if [[ -f ~/.zsh_super_snoofer_completions ]]; then
      source ~/.zsh_super_snoofer_completions
      echo "Super Snoofer completions reloaded 🐺✅"
    else
      echo "Failed to reload completions 🐺❌"
    fi
  else
    echo "Super Snoofer completions are disabled 🐺"
  fi
}

# Enable command completion for the super_snoofer command itself
if [[ "$SUPER_SNOOFER_COMPLETION_ENABLED" == "true" ]]; then
  _super_snoofer_completion() {
    local -a commands
    commands=(
      "--help:Show help message"
      "--reset_cache:Clear command cache"
      "--reset_memory:Clear cache and learned corrections"
      "--history:Show command history"
      "--frequent-typos:Show most common typos"
      "--frequent-corrections:Show most used corrections"
      "--clear-history:Clear command history"
      "--enable-history:Enable command history tracking"
      "--disable-history:Disable command history tracking"
      "--enable-completion:Enable ZSH auto-completion"
      "--disable-completion:Disable ZSH auto-completion"
      "--export-completions:Export completion script"
      "--check-command:Check if a command has typos"
      "--suggest-completion:Get real-time command suggestions"
      "--suggest-full-completion:Get full command suggestions"
      "--suggest-frequent-command:Get frequently used complete commands"
    )
    
    _describe -t commands "super_snoofer commands" commands
  }
  
  # Define the completion function
  compdef _super_snoofer_completion $SUPER_SNOOFER_CMD
fi

# Uncomment to enable the compatibility mode (fallback handler)
# command_not_found_handler() {
#   super_snoofer "$@"
#   return $?
# }

# Print status message (comment this line to disable)
echo "Super Snoofer ZSH integration loaded 🐺 (auto-suggestions, early/full completions, typo correction enabled)" 