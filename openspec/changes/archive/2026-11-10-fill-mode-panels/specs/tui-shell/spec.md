# Spec Delta

## REMOVED Requirements

### Requirement: Placeholder mode screens
**Reason**: Superseded by real per-mode content — each mode's screen is now owned and specified by its own capability (`autocrack`, `traffic-simulator`, `lessons`, `math-views`), so a shell-level placeholder requirement can only conflict with them.
**Migration**: See the four new capability specs for what each screen renders. All other `tui-shell` requirements (safe terminal lifecycle, mode navigation, tab bar, footer keybindings) are unchanged.
