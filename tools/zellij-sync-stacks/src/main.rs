use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use std::collections::BTreeMap;

#[cfg(target_arch = "wasm32")]
use zellij_tile::prelude::*;

// --- Core types for pure logic ---

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
        if !col_panes.iter().any(|p| p.pane_rows <= COLLAPSED_PANE_MAX_ROWS) {
            return None;
        }
    }

    // Sort columns left-to-right by x coordinate
    let mut sorted_cols: Vec<(usize, Vec<&PaneEntry>)> = columns.into_iter().collect();
    sorted_cols.sort_by_key(|(x, _)| *x);

    // Sort panes within each column by pane_y (top to bottom)
    let mut left: Vec<u32> = Vec::new();
    let mut right: Vec<u32> = Vec::new();

    for (i, (_, mut col_panes)) in sorted_cols.into_iter().enumerate() {
        col_panes.sort_by_key(|p| p.pane_y);
        let ids: Vec<u32> = col_panes.iter().map(|p| p.id).collect();
        if i == 0 {
            left = ids;
        } else {
            right = ids;
        }
    }

    Some(DetectedStacks { left, right })
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
    skip_updates: usize,
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
        subscribe(&[EventType::PaneUpdate, EventType::TabUpdate, EventType::PermissionRequestResult]);
        set_selectable(false);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PaneUpdate(manifest) => {
                let total_panes: usize = manifest.panes.values().map(|v| v.len()).sum();
                eprintln!("[sync-stacks] PaneUpdate: {} tabs, {} total panes, skip_updates={}",
                    manifest.panes.len(), total_panes, self.skip_updates);
                if self.skip_updates > 0 {
                    self.skip_updates -= 1;
                }
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
                eprintln!("[sync-stacks] PermissionRequestResult: {:?}", result);
            }
            _ => {}
        }
        false
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        eprintln!("[sync-stacks] pipe: name={:?}, payload={:?}", pipe_message.name, pipe_message.payload);

        if pipe_message.name == "dump" {
            self.dump_state();
            return false;
        }

        if pipe_message.name != "navigate" {
            return false;
        }

        let direction = match pipe_message.payload.as_deref() {
            Some(d @ ("up" | "down")) => d.to_string(),
            _ => return false,
        };

        let Some(ref manifest) = self.manifest else {
            eprintln!("[sync-stacks] FALLBACK: no manifest");
            self.fallback_move(&direction);
            return false;
        };

        let tab_panes = self.find_current_tab_panes(manifest);
        let Some(panes) = tab_panes else {
            eprintln!("[sync-stacks] FALLBACK: no tab found (active_tab={:?})", self.active_tab);
            self.fallback_move(&direction);
            return false;
        };

        let entries: Vec<PaneEntry> = panes.iter().map(pane_entry_from_info).collect();
        eprintln!("[sync-stacks] entries ({}):", entries.len());
        for e in &entries {
            eprintln!("  id={} x={} y={} rows={} focused={} plugin={} floating={} selectable={}",
                e.id, e.pane_x, e.pane_y, e.pane_rows, e.is_focused, e.is_plugin, e.is_floating, e.is_selectable);
        }

        let Some(stacks) = detect_stacks(&entries) else {
            eprintln!("[sync-stacks] FALLBACK: detect_stacks returned None");
            self.fallback_move(&direction);
            return false;
        };

        eprintln!("[sync-stacks] stacks: left={:?}, right={:?}", stacks.left, stacks.right);

        let Some(focused_id) = entries.iter().find(|p| p.is_focused).map(|p| p.id) else {
            eprintln!("[sync-stacks] FALLBACK: no focused pane in entries");
            self.fallback_move(&direction);
            return false;
        };

        let Some(nav) = compute_navigation(&stacks, focused_id, &direction) else {
            eprintln!("[sync-stacks] at boundary, no movement");
            return false; // at boundary, no movement
        };

        eprintln!("[sync-stacks] navigating: focus_other={}, focus_current={}", nav.focus_other, nav.focus_current);
        self.skip_updates += 2;
        focus_terminal_pane(nav.focus_other, false);
        focus_terminal_pane(nav.focus_current, false);

        false
    }
}

#[cfg(target_arch = "wasm32")]
impl SyncStacksPlugin {
    fn dump_state(&self) {
        let mut out = String::new();
        out.push_str("\n=== sync-stacks dump ===\n");

        let Some(ref manifest) = self.manifest else {
            out.push_str("manifest: None\n");
            out.push_str("=== end dump ===\n");
            write_chars(&out);
            return;
        };

        let total_panes: usize = manifest.panes.values().map(|v| v.len()).sum();
        out.push_str(&format!("manifest: Some ({} tabs, {} total panes)\n", manifest.panes.len(), total_panes));
        out.push_str(&format!("active_tab: {:?}\n", self.active_tab));
        out.push_str(&format!("skip_updates: {}\n", self.skip_updates));

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
            out.push_str(&format!(
                "  tab {}: {} panes ({} terminal, {} plugin, {} suppressed) {}\n",
                tab_idx, panes.len(), terminal_count, plugin_count, suppressed_count,
                if has_focused { "<- FOCUSED" } else { "" }
            ));
        }

        // Show detailed panes for EVERY tab
        for tab_idx in &tab_indices {
            let panes = &manifest.panes[tab_idx];
            let entries: Vec<PaneEntry> = panes.iter().map(pane_entry_from_info).collect();
            out.push_str(&format!("\ntab {} panes ({}):\n", tab_idx, entries.len()));
            for e in &entries {
                out.push_str(&format!(
                    "  id={:<4} x={:<4} y={:<4} rows={:<4} focused={:<5} plugin={:<5} floating={:<5} selectable={:<5}\n",
                    e.id, e.pane_x, e.pane_y, e.pane_rows, e.is_focused, e.is_plugin, e.is_floating, e.is_selectable
                ));
            }

            let candidates: Vec<&PaneEntry> = entries
                .iter()
                .filter(|p| p.is_selectable && !p.is_plugin && !p.is_floating)
                .collect();
            match detect_stacks(&entries) {
                Some(stacks) => {
                    out.push_str(&format!("  detect_stacks: Some(left={:?}, right={:?})\n", stacks.left, stacks.right));
                }
                None => {
                    let mut columns: HashMap<usize, Vec<&PaneEntry>> = HashMap::new();
                    for pane in &candidates {
                        columns.entry(pane.pane_x).or_default().push(pane);
                    }
                    out.push_str(&format!("  detect_stacks: None (candidates={}, columns={}, x_values={:?})\n",
                        candidates.len(), columns.len(), {
                            let mut xs: Vec<usize> = columns.keys().copied().collect();
                            xs.sort();
                            xs
                        }
                    ));
                }
            }
        }

        out.push_str("=== end dump ===\n");
        write_chars(&out);
    }

    fn find_current_tab_panes<'a>(&self, manifest: &'a PaneManifest) -> Option<&'a Vec<PaneInfo>> {
        self.active_tab.and_then(|tab| manifest.panes.get(&tab))
    }

    fn fallback_move(&self, direction: &str) {
        match direction {
            "up" => move_focus(Direction::Up),
            "down" => move_focus(Direction::Down),
            _ => {}
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
    fn make_pane(id: u32, pane_x: usize, pane_y: usize, pane_rows: usize, is_focused: bool) -> PaneEntry {
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
            make_pane(2, 0, 31, 1, false),   // left collapsed
            make_pane(3, 50, 0, 30, false),  // right expanded
            make_pane(4, 50, 31, 1, false),  // right collapsed
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
}
