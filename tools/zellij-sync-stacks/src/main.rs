use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use std::collections::BTreeMap;

#[cfg(target_arch = "wasm32")]
use std::fmt::Write as _;

#[cfg(target_arch = "wasm32")]
use zellij_tile::prelude::*;

// --- Core types for pure logic ---

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq)]
struct PaneEntry {
    id: u32,
    pane_x: usize,
    pane_y: usize,
    pane_rows: usize,
    is_plugin: bool,
    is_floating: bool,
    is_selectable: bool,
    is_focused: bool,
}

/// Max rows for a pane to be considered a collapsed stack title bar.
const COLLAPSED_PANE_MAX_ROWS: usize = 2;

#[derive(Debug, Clone, PartialEq)]
struct DetectedStacks {
    left: Vec<u32>,  // pane IDs ordered by pane_y (top to bottom)
    right: Vec<u32>, // pane IDs ordered by pane_y (top to bottom)
}

#[derive(Debug, Clone, PartialEq)]
struct NavigationResult {
    focus_other: u32,   // pane ID to focus in the other stack (first)
    focus_current: u32, // pane ID to focus in the current stack (second, restores focus)
}

#[derive(Debug, Clone, PartialEq)]
struct SyncedMoveResult {
    focused_pane_id: u32,
    other_pane_id: u32,
}

// --- Pure functions ---

#[cfg(target_arch = "wasm32")]
fn pane_entry_from_info(info: &PaneInfo) -> PaneEntry {
    PaneEntry {
        id: info.id,
        pane_x: info.pane_x,
        pane_y: info.pane_y,
        pane_rows: info.pane_rows,
        is_plugin: info.is_plugin,
        is_floating: info.is_floating,
        is_selectable: info.is_selectable,
        is_focused: info.is_focused,
    }
}

fn detect_stacks(panes: &[PaneEntry]) -> Option<DetectedStacks> {
    // Filter to selectable, non-plugin, non-floating terminal panes
    let candidates: Vec<&PaneEntry> = panes
        .iter()
        .filter(|p| p.is_selectable && !p.is_plugin && !p.is_floating)
        .collect();

    // Group by pane_x
    let mut columns: HashMap<usize, Vec<&PaneEntry>> = HashMap::new();
    for pane in &candidates {
        columns.entry(pane.pane_x).or_default().push(pane);
    }

    // Need exactly 2 columns
    if columns.len() != 2 {
        return None;
    }

    // Each column needs 2+ panes, and at least one must be collapsed (small height)
    // to distinguish stacked layouts from grids
    for col_panes in columns.values() {
        if col_panes.len() < 2 {
            return None;
        }
        if !col_panes
            .iter()
            .any(|p| p.pane_rows <= COLLAPSED_PANE_MAX_ROWS)
        {
            return None;
        }
    }

    // Sort columns left-to-right by x coordinate
    let mut sorted_cols: Vec<(usize, Vec<&PaneEntry>)> = columns.into_iter().collect();
    sorted_cols.sort_by_key(|(x, _)| *x);

    // Sort panes within each column by pane_y (top to bottom)
    for (_, col_panes) in &mut sorted_cols {
        col_panes.sort_by_key(|p| p.pane_y);
    }

    let mut cols = sorted_cols
        .into_iter()
        .map(|(_, col)| col.iter().map(|p| p.id).collect());
    let left = cols.next().unwrap();
    let right = cols.next().unwrap();

    Some(DetectedStacks { left, right })
}

fn pane_count_for_new(panes: &[PaneEntry]) -> usize {
    if detect_stacks(panes).is_some() {
        2
    } else {
        1
    }
}

fn compute_synced_move(
    stacks: &DetectedStacks,
    focused_id: u32,
    direction: &str,
) -> Option<SyncedMoveResult> {
    // Only handle up/down — left/right fall back to normal move
    if direction != "up" && direction != "down" {
        return None;
    }

    let (current_stack, other_stack, current_idx) =
        if let Some(idx) = stacks.left.iter().position(|&id| id == focused_id) {
            (&stacks.left, &stacks.right, idx)
        } else if let Some(idx) = stacks.right.iter().position(|&id| id == focused_id) {
            (&stacks.right, &stacks.left, idx)
        } else {
            return None;
        };

    // Check focused pane can move in this direction
    match direction {
        "down" => {
            if current_idx + 1 >= current_stack.len() {
                return None;
            }
        }
        "up" => {
            if current_idx == 0 {
                return None;
            }
        }
        _ => return None,
    }

    // Counterpart is at the same index as the focused pane (before move)
    let other_idx = current_idx.min(other_stack.len() - 1);

    // Check that counterpart can also move in this direction
    match direction {
        "down" => {
            if other_idx + 1 >= other_stack.len() {
                return None;
            }
        }
        "up" => {
            if other_idx == 0 {
                return None;
            }
        }
        _ => return None,
    }

    Some(SyncedMoveResult {
        focused_pane_id: current_stack[current_idx],
        other_pane_id: other_stack[other_idx],
    })
}

fn compute_navigation(
    stacks: &DetectedStacks,
    focused_id: u32,
    direction: &str,
) -> Option<NavigationResult> {
    let (current_stack, other_stack, current_idx) =
        if let Some(idx) = stacks.left.iter().position(|&id| id == focused_id) {
            (&stacks.left, &stacks.right, idx)
        } else if let Some(idx) = stacks.right.iter().position(|&id| id == focused_id) {
            (&stacks.right, &stacks.left, idx)
        } else {
            return None;
        };

    let target_idx = match direction {
        "down" => {
            if current_idx + 1 >= current_stack.len() {
                return None; // at bottom
            }
            current_idx + 1
        }
        "up" => {
            if current_idx == 0 {
                return None; // at top
            }
            current_idx - 1
        }
        _ => return None,
    };

    let other_target_idx = target_idx.min(other_stack.len() - 1);

    Some(NavigationResult {
        focus_other: other_stack[other_target_idx],
        focus_current: current_stack[target_idx],
    })
}

// --- Plugin state and glue (WASM only) ---

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct SyncStacksPlugin {
    manifest: Option<PaneManifest>,
    active_tab: Option<usize>,
}

#[cfg(target_arch = "wasm32")]
impl ZellijPlugin for SyncStacksPlugin {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        eprintln!("[sync-stacks] load: requesting permissions");
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::WriteToStdin,
        ]);
        subscribe(&[
            EventType::PaneUpdate,
            EventType::TabUpdate,
            EventType::PermissionRequestResult,
        ]);
        set_selectable(false);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PaneUpdate(manifest) => {
                let total_panes: usize = manifest.panes.values().map(Vec::len).sum();
                eprintln!(
                    "[sync-stacks] PaneUpdate: {} tabs, {} total panes",
                    manifest.panes.len(),
                    total_panes,
                );
                self.manifest = Some(manifest);
            }
            Event::TabUpdate(tabs) => {
                let prev = self.active_tab;
                self.active_tab = tabs.iter().find(|t| t.active).map(|t| t.position);
                if self.active_tab != prev {
                    eprintln!("[sync-stacks] TabUpdate: active_tab={:?}", self.active_tab);
                }
            }
            Event::PermissionRequestResult(result) => {
                eprintln!("[sync-stacks] PermissionRequestResult: {result:?}");
            }
            _ => {}
        }
        false
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        eprintln!(
            "[sync-stacks] pipe: name={:?}, payload={:?}",
            pipe_message.name, pipe_message.payload
        );

        match pipe_message.name.as_str() {
            "dump" => self.dump_state(),
            "navigate" => self.handle_navigate(pipe_message.payload.as_deref()),
            "new_pane" => self.handle_new_pane(),
            "move_pane" => self.handle_move_pane(pipe_message.payload.as_deref()),
            _ => {}
        }

        false
    }
}

#[cfg(target_arch = "wasm32")]
impl SyncStacksPlugin {
    fn handle_navigate(&self, payload: Option<&str>) {
        let direction = match payload {
            Some(d @ ("up" | "down")) => d.to_string(),
            _ => return,
        };

        let Some(ref manifest) = self.manifest else {
            eprintln!("[sync-stacks] FALLBACK: no manifest");
            Self::fallback_focus(&direction);
            return;
        };

        let Some(panes) = self.find_current_tab_panes(manifest) else {
            eprintln!(
                "[sync-stacks] FALLBACK: no tab found (active_tab={:?})",
                self.active_tab
            );
            Self::fallback_focus(&direction);
            return;
        };

        let entries: Vec<PaneEntry> = panes.iter().map(pane_entry_from_info).collect();
        Self::log_entries(&entries);

        let Some(stacks) = detect_stacks(&entries) else {
            eprintln!("[sync-stacks] FALLBACK: detect_stacks returned None");
            Self::fallback_focus(&direction);
            return;
        };

        eprintln!(
            "[sync-stacks] stacks: left={:?}, right={:?}",
            stacks.left, stacks.right
        );

        let Some(focused_id) = entries.iter().find(|p| p.is_focused).map(|p| p.id) else {
            eprintln!("[sync-stacks] FALLBACK: no focused pane in entries");
            Self::fallback_focus(&direction);
            return;
        };

        let Some(nav) = compute_navigation(&stacks, focused_id, &direction) else {
            eprintln!("[sync-stacks] at boundary, no movement");
            return;
        };

        eprintln!(
            "[sync-stacks] navigating: focus_other={}, focus_current={}",
            nav.focus_other, nav.focus_current
        );
        focus_terminal_pane(nav.focus_other, false);
        focus_terminal_pane(nav.focus_current, false);
    }

    fn handle_new_pane(&self) {
        let count = self
            .manifest
            .as_ref()
            .and_then(|m| self.find_current_tab_panes(m))
            .map(|panes| {
                let entries: Vec<PaneEntry> = panes.iter().map(pane_entry_from_info).collect();
                pane_count_for_new(&entries)
            })
            .unwrap_or(1);

        eprintln!("[sync-stacks] new_pane: opening {count} terminal(s)");
        for _ in 0..count {
            open_terminal(std::path::Path::new("."));
        }
    }

    fn handle_move_pane(&self, payload: Option<&str>) {
        let Some(direction) = payload else {
            return;
        };

        // Left/right: fall back to normal directional move immediately
        if direction == "left" || direction == "right" {
            eprintln!("[sync-stacks] move_pane: {direction} → fallback");
            Self::fallback_move_pane(direction);
            return;
        }

        if direction != "up" && direction != "down" {
            return;
        }

        let Some(ref manifest) = self.manifest else {
            eprintln!("[sync-stacks] move_pane FALLBACK: no manifest");
            Self::fallback_move_pane(direction);
            return;
        };

        let Some(panes) = self.find_current_tab_panes(manifest) else {
            eprintln!("[sync-stacks] move_pane FALLBACK: no tab");
            Self::fallback_move_pane(direction);
            return;
        };

        let entries: Vec<PaneEntry> = panes.iter().map(pane_entry_from_info).collect();

        let Some(stacks) = detect_stacks(&entries) else {
            eprintln!("[sync-stacks] move_pane FALLBACK: no stacks");
            Self::fallback_move_pane(direction);
            return;
        };

        let Some(focused_id) = entries.iter().find(|p| p.is_focused).map(|p| p.id) else {
            eprintln!("[sync-stacks] move_pane FALLBACK: no focused pane");
            Self::fallback_move_pane(direction);
            return;
        };

        let Some(result) = compute_synced_move(&stacks, focused_id, direction) else {
            eprintln!("[sync-stacks] move_pane FALLBACK: compute returned None");
            Self::fallback_move_pane(direction);
            return;
        };

        let dir = Self::parse_direction(direction).unwrap();
        eprintln!(
            "[sync-stacks] move_pane: moving {} and {} {:?}",
            result.focused_pane_id, result.other_pane_id, dir
        );
        move_pane_with_pane_id_in_direction(PaneId::Terminal(result.focused_pane_id), dir);
        move_pane_with_pane_id_in_direction(PaneId::Terminal(result.other_pane_id), dir);
        focus_terminal_pane(result.focused_pane_id, false);
    }

    fn log_entries(entries: &[PaneEntry]) {
        eprintln!("[sync-stacks] entries ({}):", entries.len());
        for e in entries {
            eprintln!(
                "  id={} x={} y={} rows={} focused={} plugin={} floating={} selectable={}",
                e.id,
                e.pane_x,
                e.pane_y,
                e.pane_rows,
                e.is_focused,
                e.is_plugin,
                e.is_floating,
                e.is_selectable
            );
        }
    }

    fn dump_state(&self) {
        let mut out = String::new();
        out.push_str("\n=== sync-stacks dump ===\n");

        let Some(ref manifest) = self.manifest else {
            out.push_str("manifest: None\n");
            out.push_str("=== end dump ===\n");
            write_chars(&out);
            return;
        };

        let total_panes: usize = manifest.panes.values().map(Vec::len).sum();
        let _ = writeln!(
            out,
            "manifest: Some ({} tabs, {total_panes} total panes)",
            manifest.panes.len(),
        );
        let _ = writeln!(out, "active_tab: {:?}", self.active_tab);

        // Show ALL tabs summary
        out.push_str("\nall tabs:\n");
        let mut tab_indices: Vec<&usize> = manifest.panes.keys().collect();
        tab_indices.sort();
        for tab_idx in &tab_indices {
            let panes = &manifest.panes[tab_idx];
            let has_focused = panes.iter().any(|p| p.is_focused);
            let terminal_count = panes.iter().filter(|p| !p.is_plugin).count();
            let plugin_count = panes.iter().filter(|p| p.is_plugin).count();
            let suppressed_count = panes.iter().filter(|p| p.is_suppressed).count();
            let _ = writeln!(
                out,
                "  tab {tab_idx}: {} panes ({terminal_count} terminal, {plugin_count} plugin, {suppressed_count} suppressed) {}",
                panes.len(),
                if has_focused { "<- FOCUSED" } else { "" }
            );
        }

        // Show detailed panes for EVERY tab
        for tab_idx in &tab_indices {
            let panes = &manifest.panes[tab_idx];
            let entries: Vec<PaneEntry> = panes.iter().map(pane_entry_from_info).collect();
            let _ = writeln!(out, "\ntab {tab_idx} panes ({}):", entries.len());
            for e in &entries {
                let _ = writeln!(
                    out,
                    "  id={:<4} x={:<4} y={:<4} rows={:<4} focused={:<5} plugin={:<5} floating={:<5} selectable={:<5}",
                    e.id, e.pane_x, e.pane_y, e.pane_rows, e.is_focused, e.is_plugin, e.is_floating, e.is_selectable
                );
            }

            let candidates: Vec<&PaneEntry> = entries
                .iter()
                .filter(|p| p.is_selectable && !p.is_plugin && !p.is_floating)
                .collect();
            if let Some(stacks) = detect_stacks(&entries) {
                let _ = writeln!(
                    out,
                    "  detect_stacks: Some(left={:?}, right={:?})",
                    stacks.left, stacks.right
                );
            } else {
                let mut columns: HashMap<usize, Vec<&PaneEntry>> = HashMap::new();
                for pane in &candidates {
                    columns.entry(pane.pane_x).or_default().push(pane);
                }
                let mut xs: Vec<usize> = columns.keys().copied().collect();
                xs.sort_unstable();
                let _ = writeln!(
                    out,
                    "  detect_stacks: None (candidates={}, columns={}, x_values={xs:?})",
                    candidates.len(),
                    columns.len(),
                );
            }
        }

        out.push_str("=== end dump ===\n");
        write_chars(&out);
    }

    fn find_current_tab_panes<'a>(&self, manifest: &'a PaneManifest) -> Option<&'a Vec<PaneInfo>> {
        self.active_tab.and_then(|tab| manifest.panes.get(&tab))
    }

    fn parse_direction(direction: &str) -> Option<Direction> {
        match direction {
            "up" => Some(Direction::Up),
            "down" => Some(Direction::Down),
            "left" => Some(Direction::Left),
            "right" => Some(Direction::Right),
            _ => None,
        }
    }

    fn fallback_focus(direction: &str) {
        if let Some(dir) = Self::parse_direction(direction) {
            move_focus(dir);
        }
    }

    fn fallback_move_pane(direction: &str) {
        if let Some(dir) = Self::parse_direction(direction) {
            move_pane_with_direction(dir);
        }
    }
}

#[cfg(target_arch = "wasm32")]
register_plugin!(SyncStacksPlugin);

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a pane entry for testing.
    /// `pane_rows=1` means collapsed (title bar only), `pane_rows=30` means expanded.
    fn make_pane(
        id: u32,
        pane_x: usize,
        pane_y: usize,
        pane_rows: usize,
        is_focused: bool,
    ) -> PaneEntry {
        PaneEntry {
            id,
            pane_x,
            pane_y,
            pane_rows,
            is_plugin: false,
            is_floating: false,
            is_selectable: true,
            is_focused,
        }
    }

    #[test]
    fn detect_two_stacks() {
        // Two columns, each with one expanded pane and one collapsed
        let panes = vec![
            make_pane(1, 0, 0, 30, true),   // left expanded
            make_pane(2, 0, 31, 1, false),  // left collapsed
            make_pane(3, 50, 0, 30, false), // right expanded
            make_pane(4, 50, 31, 1, false), // right collapsed
        ];

        let result = detect_stacks(&panes).unwrap();
        assert_eq!(result.left, vec![1, 2]);
        assert_eq!(result.right, vec![3, 4]);
    }

    #[test]
    fn detect_stacks_single_column_returns_none() {
        let panes = vec![
            make_pane(1, 0, 0, 30, true),
            make_pane(2, 0, 31, 1, false),
            make_pane(3, 0, 32, 1, false),
        ];

        assert!(detect_stacks(&panes).is_none());
    }

    #[test]
    fn detect_stacks_grid_returns_none() {
        // 2x2 grid: all panes have significant height — not stacked
        let panes = vec![
            make_pane(1, 0, 0, 15, true),
            make_pane(2, 0, 16, 15, false),
            make_pane(3, 50, 0, 15, false),
            make_pane(4, 50, 16, 15, false),
        ];

        assert!(detect_stacks(&panes).is_none());
    }

    #[test]
    fn detect_stacks_filters_plugins_and_floating() {
        let panes = vec![
            make_pane(1, 0, 0, 30, true),
            make_pane(2, 0, 31, 1, false),
            make_pane(3, 50, 0, 30, false),
            make_pane(4, 50, 31, 1, false),
            PaneEntry {
                id: 99,
                pane_x: 0,
                pane_y: 0,
                pane_rows: 1,
                is_plugin: true,
                is_floating: false,
                is_selectable: true,
                is_focused: false,
            },
            PaneEntry {
                id: 100,
                pane_x: 50,
                pane_y: 0,
                pane_rows: 30,
                is_plugin: false,
                is_floating: true,
                is_selectable: true,
                is_focused: false,
            },
        ];

        let result = detect_stacks(&panes).unwrap();
        assert_eq!(result.left, vec![1, 2]);
        assert_eq!(result.right, vec![3, 4]);
    }

    #[test]
    fn detect_stacks_orders_by_pane_y() {
        // Panes provided out of order — should be sorted by y
        let panes = vec![
            make_pane(2, 0, 31, 1, false),
            make_pane(1, 0, 0, 30, true),
            make_pane(4, 50, 31, 1, false),
            make_pane(3, 50, 0, 30, false),
        ];

        let result = detect_stacks(&panes).unwrap();
        assert_eq!(result.left, vec![1, 2]);
        assert_eq!(result.right, vec![3, 4]);
    }

    #[test]
    fn navigate_down() {
        let stacks = DetectedStacks {
            left: vec![1, 2, 3],
            right: vec![4, 5, 6],
        };

        let nav = compute_navigation(&stacks, 1, "down").unwrap();
        assert_eq!(nav.focus_current, 2);
        assert_eq!(nav.focus_other, 5);
    }

    #[test]
    fn navigate_up() {
        let stacks = DetectedStacks {
            left: vec![1, 2, 3],
            right: vec![4, 5, 6],
        };

        let nav = compute_navigation(&stacks, 3, "up").unwrap();
        assert_eq!(nav.focus_current, 2);
        assert_eq!(nav.focus_other, 5);
    }

    #[test]
    fn navigate_down_at_bottom_returns_none() {
        let stacks = DetectedStacks {
            left: vec![1, 2],
            right: vec![3, 4],
        };

        assert!(compute_navigation(&stacks, 2, "down").is_none());
    }

    #[test]
    fn navigate_up_at_top_returns_none() {
        let stacks = DetectedStacks {
            left: vec![1, 2],
            right: vec![3, 4],
        };

        assert!(compute_navigation(&stacks, 1, "up").is_none());
    }

    #[test]
    fn navigate_unequal_stacks_clamps() {
        let stacks = DetectedStacks {
            left: vec![1, 2],
            right: vec![3, 4, 5, 6],
        };

        let nav = compute_navigation(&stacks, 5, "down").unwrap();
        assert_eq!(nav.focus_current, 6);
        assert_eq!(nav.focus_other, 2);
    }

    // --- detect_stacks edge cases ---

    #[test]
    fn detect_stacks_three_columns_returns_none() {
        let panes = vec![
            make_pane(1, 0, 0, 30, true),
            make_pane(2, 0, 31, 1, false),
            make_pane(3, 50, 0, 30, false),
            make_pane(4, 50, 31, 1, false),
            make_pane(5, 100, 0, 30, false),
            make_pane(6, 100, 31, 1, false),
        ];

        assert!(detect_stacks(&panes).is_none());
    }

    #[test]
    fn detect_stacks_one_pane_in_column_returns_none() {
        let panes = vec![
            make_pane(1, 0, 0, 30, true),
            make_pane(2, 0, 31, 1, false),
            make_pane(3, 50, 0, 30, false), // only one pane in right column
        ];

        assert!(detect_stacks(&panes).is_none());
    }

    #[test]
    fn detect_stacks_collapsed_boundary() {
        // pane_rows=2 is exactly COLLAPSED_PANE_MAX_ROWS — should count as collapsed
        let panes_at_boundary = vec![
            make_pane(1, 0, 0, 30, true),
            make_pane(2, 0, 31, 2, false),
            make_pane(3, 50, 0, 30, false),
            make_pane(4, 50, 31, 2, false),
        ];
        assert!(detect_stacks(&panes_at_boundary).is_some());

        // pane_rows=3 is above the threshold — not collapsed, so no stacks detected
        let panes_above = vec![
            make_pane(1, 0, 0, 30, true),
            make_pane(2, 0, 31, 3, false),
            make_pane(3, 50, 0, 30, false),
            make_pane(4, 50, 31, 3, false),
        ];
        assert!(detect_stacks(&panes_above).is_none());
    }

    #[test]
    fn detect_stacks_filters_non_selectable() {
        let panes = vec![
            make_pane(1, 0, 0, 30, true),
            make_pane(2, 0, 31, 1, false),
            make_pane(3, 50, 0, 30, false),
            make_pane(4, 50, 31, 1, false),
            PaneEntry {
                id: 99,
                pane_x: 100,
                pane_y: 0,
                pane_rows: 30,
                is_plugin: false,
                is_floating: false,
                is_selectable: false,
                is_focused: false,
            },
        ];

        let result = detect_stacks(&panes).unwrap();
        assert_eq!(result.left, vec![1, 2]);
        assert_eq!(result.right, vec![3, 4]);
    }

    #[test]
    fn detect_stacks_empty_returns_none() {
        assert!(detect_stacks(&[]).is_none());
    }

    // --- compute_navigation edge cases ---

    #[test]
    fn navigate_down_from_right_stack() {
        let stacks = DetectedStacks {
            left: vec![1, 2, 3],
            right: vec![4, 5, 6],
        };

        let nav = compute_navigation(&stacks, 4, "down").unwrap();
        assert_eq!(nav.focus_current, 5);
        assert_eq!(nav.focus_other, 2);
    }

    #[test]
    fn navigate_invalid_direction_returns_none() {
        let stacks = DetectedStacks {
            left: vec![1, 2],
            right: vec![3, 4],
        };

        assert!(compute_navigation(&stacks, 1, "left").is_none());
    }

    #[test]
    fn navigate_focused_not_in_stacks_returns_none() {
        let stacks = DetectedStacks {
            left: vec![1, 2],
            right: vec![3, 4],
        };

        assert!(compute_navigation(&stacks, 99, "down").is_none());
    }

    #[test]
    fn navigate_down_from_middle() {
        let stacks = DetectedStacks {
            left: vec![1, 2, 3],
            right: vec![4, 5, 6],
        };

        let nav = compute_navigation(&stacks, 2, "down").unwrap();
        assert_eq!(nav.focus_current, 3);
        assert_eq!(nav.focus_other, 6);
    }

    // --- pane_count_for_new tests ---

    #[test]
    fn new_pane_count_two_column_stacked() {
        let panes = vec![
            make_pane(1, 0, 0, 30, true),
            make_pane(2, 0, 31, 1, false),
            make_pane(3, 50, 0, 30, false),
            make_pane(4, 50, 31, 1, false),
        ];
        assert_eq!(pane_count_for_new(&panes), 2);
    }

    #[test]
    fn new_pane_count_single_column() {
        let panes = vec![
            make_pane(1, 0, 0, 30, true),
            make_pane(2, 0, 31, 1, false),
        ];
        assert_eq!(pane_count_for_new(&panes), 1);
    }

    #[test]
    fn new_pane_count_grid() {
        let panes = vec![
            make_pane(1, 0, 0, 15, true),
            make_pane(2, 0, 16, 15, false),
            make_pane(3, 50, 0, 15, false),
            make_pane(4, 50, 16, 15, false),
        ];
        assert_eq!(pane_count_for_new(&panes), 1);
    }

    #[test]
    fn new_pane_count_empty() {
        assert_eq!(pane_count_for_new(&[]), 1);
    }

    // --- compute_synced_move tests ---

    #[test]
    fn synced_move_down_from_top_left() {
        let stacks = DetectedStacks {
            left: vec![1, 2, 3],
            right: vec![4, 5, 6],
        };
        let result = compute_synced_move(&stacks, 1, "down").unwrap();
        assert_eq!(result.focused_pane_id, 1);
        assert_eq!(result.other_pane_id, 4);
    }

    #[test]
    fn synced_move_up_from_bottom_right() {
        let stacks = DetectedStacks {
            left: vec![1, 2, 3],
            right: vec![4, 5, 6],
        };
        let result = compute_synced_move(&stacks, 6, "up").unwrap();
        assert_eq!(result.focused_pane_id, 6);
        assert_eq!(result.other_pane_id, 3);
    }

    #[test]
    fn synced_move_down_at_bottom_returns_none() {
        let stacks = DetectedStacks {
            left: vec![1, 2],
            right: vec![3, 4],
        };
        assert!(compute_synced_move(&stacks, 2, "down").is_none());
    }

    #[test]
    fn synced_move_up_at_top_returns_none() {
        let stacks = DetectedStacks {
            left: vec![1, 2],
            right: vec![3, 4],
        };
        assert!(compute_synced_move(&stacks, 1, "up").is_none());
    }

    #[test]
    fn synced_move_left_right_returns_none() {
        let stacks = DetectedStacks {
            left: vec![1, 2],
            right: vec![3, 4],
        };
        assert!(compute_synced_move(&stacks, 1, "left").is_none());
        assert!(compute_synced_move(&stacks, 1, "right").is_none());
    }

    #[test]
    fn synced_move_focused_not_in_stacks_returns_none() {
        let stacks = DetectedStacks {
            left: vec![1, 2],
            right: vec![3, 4],
        };
        assert!(compute_synced_move(&stacks, 99, "down").is_none());
    }

    #[test]
    fn synced_move_unequal_stacks_counterpart_at_boundary() {
        let stacks = DetectedStacks {
            left: vec![1, 2, 3],
            right: vec![4, 5],
        };
        // Focused on pane 3 (left, idx 2), counterpart clamped to idx 2 → but right only has idx 0,1
        // So counterpart is at idx 2 clamped to 1 (pane 5). Moving up: counterpart at idx 1, can move up. OK.
        // But moving down from idx 2: focused can't move (at bottom). Returns None.
        assert!(compute_synced_move(&stacks, 3, "down").is_none());

        // Focused on pane 2 (left, idx 1), counterpart at idx 1 (pane 5).
        // Moving down: focused can go to idx 2, counterpart at idx 1 can't go to idx 2 (only 2 items).
        assert!(compute_synced_move(&stacks, 2, "down").is_none());
    }

    #[test]
    fn synced_move_from_middle() {
        let stacks = DetectedStacks {
            left: vec![1, 2, 3],
            right: vec![4, 5, 6],
        };
        let result = compute_synced_move(&stacks, 2, "down").unwrap();
        assert_eq!(result.focused_pane_id, 2);
        assert_eq!(result.other_pane_id, 5);

        let result = compute_synced_move(&stacks, 5, "up").unwrap();
        assert_eq!(result.focused_pane_id, 5);
        assert_eq!(result.other_pane_id, 2);
    }
}
