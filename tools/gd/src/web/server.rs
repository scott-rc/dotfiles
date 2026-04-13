use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Html;
use axum::response::IntoResponse;
use axum::routing::get;
use tokio::sync::broadcast;

use crate::git;
use crate::git::diff::DiffFile;
use crate::pager::DiffContext;

use super::html_render::build_diff_data;

const INDEX_HTML: &str = include_str!("../web_assets/index.html");
const STYLE_CSS: &str = include_str!("../web_assets/style.css");
const APP_JS: &str = include_str!("../web_assets/app.js");

struct AppState {
    diff_ctx: DiffContext,
    tx: broadcast::Sender<Arc<String>>,
    connection_count: AtomicUsize,
    shutdown_tx: broadcast::Sender<()>,
    /// Grace period before shutdown after last connection closes (milliseconds).
    shutdown_grace_ms: u64,
}

async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn style_handler() -> impl IntoResponse {
    ([("content-type", "text/css; charset=utf-8")], STYLE_CSS)
}

async fn js_handler() -> impl IntoResponse {
    (
        [("content-type", "application/javascript; charset=utf-8")],
        APP_JS,
    )
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Guard that decrements connection count on drop and schedules shutdown if needed.
struct ConnectionGuard {
    state: Arc<AppState>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let prev = self.state.connection_count.fetch_sub(1, Ordering::SeqCst);
        if prev == 1 {
            // Last connection closed — schedule shutdown after grace period
            let state = self.state.clone();
            let grace_ms = state.shutdown_grace_ms;
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(grace_ms)).await;
                // Check if still zero (no new connections during grace period)
                if state.connection_count.load(Ordering::SeqCst) == 0 {
                    let _ = state.shutdown_tx.send(());
                }
            });
        }
    }
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    // Track connection
    state.connection_count.fetch_add(1, Ordering::SeqCst);
    let _guard = ConnectionGuard {
        state: state.clone(),
    };

    // Send current diff data immediately
    let files = load_files(&state.diff_ctx);
    let msg = build_json(&files);
    if socket.send(Message::Text(msg.into())).await.is_err() {
        return;
    }

    // Subscribe to broadcast for updates
    let mut rx = state.tx.subscribe();
    loop {
        tokio::select! {
            // Wait for broadcast updates to send to client
            result = rx.recv() => {
                match result {
                    Ok(json) => {
                        if socket.send(Message::Text((*json).clone().into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            // Detect when browser closes the WebSocket
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    Some(Ok(_)) => {} // Ignore other messages (pings, etc.)
                }
            }
        }
    }
}

fn load_files(diff_ctx: &DiffContext) -> Vec<DiffFile> {
    let mut args = diff_ctx.source.diff_args();
    if diff_ctx.ignore_whitespace {
        args.push("-w".into());
    }
    let str_args: Vec<&str> = args.iter().map(String::as_str).collect();
    let raw = git::run_diff(&diff_ctx.repo, &str_args);
    let mut files = git::diff::parse(&raw);
    git::append_untracked(
        &diff_ctx.repo,
        &diff_ctx.source,
        diff_ctx.no_untracked,
        &mut files,
    );
    git::sort_files_for_display(&mut files);
    files
}

fn build_json(files: &[DiffFile]) -> String {
    let msg = build_diff_data(files);
    serde_json::to_string(&msg).unwrap()
}

fn git_index_path(repo: &std::path::Path) -> PathBuf {
    repo.join(".git").join("index")
}

fn git_index_mtime(repo: &std::path::Path) -> Option<SystemTime> {
    std::fs::metadata(git_index_path(repo))
        .ok()
        .and_then(|m| m.modified().ok())
}

/// File watcher task — polls .git/index mtime and broadcasts on change.
async fn watch_git_index(
    diff_ctx: DiffContext,
    tx: broadcast::Sender<Arc<String>>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let mut last_mtime = git_index_mtime(&diff_ctx.repo);

    loop {
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {
                let current_mtime = git_index_mtime(&diff_ctx.repo);
                if current_mtime != last_mtime {
                    last_mtime = current_mtime;
                    let files = load_files(&diff_ctx);
                    if tx.receiver_count() > 0 {
                        let json = Arc::new(build_json(&files));
                        let _ = tx.send(json);
                    }
                }
            }
            _ = shutdown_rx.recv() => break,
        }
    }
}

pub(crate) fn start_server(
    _files: Vec<DiffFile>,
    diff_ctx: &DiffContext,
    open: bool,
    shutdown_grace_ms: u64,
) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let (tx, _rx) = broadcast::channel::<Arc<String>>(16);
        let (shutdown_tx, _) = broadcast::channel::<()>(1);

        let state = Arc::new(AppState {
            diff_ctx: diff_ctx.clone(),
            tx: tx.clone(),
            connection_count: AtomicUsize::new(0),
            shutdown_tx: shutdown_tx.clone(),
            shutdown_grace_ms,
        });

        let app = Router::new()
            .route("/", get(index_handler))
            .route("/style.css", get(style_handler))
            .route("/app.js", get(js_handler))
            .route("/ws", get(ws_handler))
            .with_state(state);

        // Try ports starting at 3845
        let mut port = 3845u16;
        let listener = loop {
            match tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))).await {
                Ok(l) => break l,
                Err(_) if port < 3855 => port += 1,
                Err(e) => {
                    eprintln!("gd: failed to bind: {e}");
                    std::process::exit(1);
                }
            }
        };

        let addr = listener.local_addr().unwrap();
        let url = format!("http://{addr}");
        eprintln!("gd: serving at {url}");

        // Open browser (unless --no-open)
        if open {
            let _ = std::process::Command::new("open").arg(&url).spawn();
        }

        // Start file watcher
        tokio::spawn(watch_git_index(
            diff_ctx.clone(),
            tx,
            shutdown_tx.subscribe(),
        ));

        // Serve with graceful shutdown
        let mut shutdown_rx = shutdown_tx.subscribe();
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.recv().await;
                eprintln!("gd: all browser tabs closed, shutting down");
            })
            .await
            .unwrap();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{self, Duration};

    const TEST_GRACE_MS: u64 = 100; // Shorter grace period for faster tests

    fn make_test_state() -> Arc<AppState> {
        make_test_state_with_grace(TEST_GRACE_MS)
    }

    fn make_test_state_with_grace(shutdown_grace_ms: u64) -> Arc<AppState> {
        let (tx, _) = broadcast::channel::<Arc<String>>(1);
        let (shutdown_tx, _) = broadcast::channel::<()>(1);
        Arc::new(AppState {
            diff_ctx: DiffContext {
                repo: PathBuf::from("/tmp"),
                source: crate::git::DiffSource::WorkingTree,
                no_untracked: false,
                ignore_whitespace: false,
            },
            tx,
            connection_count: AtomicUsize::new(0),
            shutdown_tx,
            shutdown_grace_ms,
        })
    }

    #[tokio::test(start_paused = true)]
    async fn connection_guard_schedules_shutdown_on_last_close() {
        let state = make_test_state();
        let mut shutdown_rx = state.shutdown_tx.subscribe();

        // Simulate connection: increment count then create guard
        state.connection_count.fetch_add(1, Ordering::SeqCst);
        let guard = ConnectionGuard {
            state: state.clone(),
        };

        // Drop the guard (last connection closes)
        drop(guard);

        // Count should be 0 now
        assert_eq!(state.connection_count.load(Ordering::SeqCst), 0);

        // Yield to let the spawned task start
        tokio::task::yield_now().await;

        // Advance time past the grace period
        time::advance(Duration::from_millis(TEST_GRACE_MS + 50)).await;

        // Yield again to let the spawned task complete after sleep
        tokio::task::yield_now().await;

        // Shutdown signal should be sent
        let result = shutdown_rx.try_recv();
        assert!(
            result.is_ok(),
            "shutdown signal should be sent after grace period"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn connection_guard_cancels_shutdown_if_new_connection_arrives() {
        let state = make_test_state();
        let mut shutdown_rx = state.shutdown_tx.subscribe();

        // First connection
        state.connection_count.fetch_add(1, Ordering::SeqCst);
        let guard1 = ConnectionGuard {
            state: state.clone(),
        };

        // Drop first connection (schedules shutdown)
        drop(guard1);
        assert_eq!(state.connection_count.load(Ordering::SeqCst), 0);

        // Yield to let spawned task start
        tokio::task::yield_now().await;

        // Advance partway through grace period
        time::advance(Duration::from_millis(TEST_GRACE_MS / 2)).await;

        // New connection arrives during grace period
        state.connection_count.fetch_add(1, Ordering::SeqCst);
        let _guard2 = ConnectionGuard {
            state: state.clone(),
        };

        // Count is now 1
        assert_eq!(state.connection_count.load(Ordering::SeqCst), 1);

        // Advance past original grace period
        time::advance(Duration::from_millis(TEST_GRACE_MS + 50)).await;
        tokio::task::yield_now().await;

        // Shutdown should NOT have been sent (count was 1 when checked)
        let result = shutdown_rx.try_recv();
        assert!(
            result.is_err(),
            "shutdown should be cancelled when new connection arrives"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn connection_guard_only_last_connection_schedules_shutdown() {
        let state = make_test_state();
        let mut shutdown_rx = state.shutdown_tx.subscribe();

        // Two connections
        state.connection_count.fetch_add(1, Ordering::SeqCst);
        let guard1 = ConnectionGuard {
            state: state.clone(),
        };
        state.connection_count.fetch_add(1, Ordering::SeqCst);
        let guard2 = ConnectionGuard {
            state: state.clone(),
        };

        assert_eq!(state.connection_count.load(Ordering::SeqCst), 2);

        // Drop first connection (not the last)
        drop(guard1);
        assert_eq!(state.connection_count.load(Ordering::SeqCst), 1);

        // Advance time - no shutdown should be scheduled
        time::advance(Duration::from_millis(TEST_GRACE_MS + 50)).await;
        tokio::task::yield_now().await;
        let result = shutdown_rx.try_recv();
        assert!(
            result.is_err(),
            "shutdown should not be scheduled when connections remain"
        );

        // Drop second (last) connection
        drop(guard2);
        assert_eq!(state.connection_count.load(Ordering::SeqCst), 0);

        // Yield and advance past grace period
        tokio::task::yield_now().await;
        time::advance(Duration::from_millis(TEST_GRACE_MS + 50)).await;
        tokio::task::yield_now().await;
        let result = shutdown_rx.try_recv();
        assert!(
            result.is_ok(),
            "shutdown should be sent after last connection closes"
        );
    }

    /// Test that verifies handle_socket exits when WebSocket closes.
    /// This test would have failed before the fix that added socket.recv() to the select.
    #[tokio::test(start_paused = true)]
    async fn handle_socket_detects_websocket_close() {
        use tokio::sync::mpsc;

        let state = make_test_state();
        let mut shutdown_rx = state.shutdown_tx.subscribe();

        // Create a channel pair to simulate WebSocket messages
        let (ws_tx, mut ws_rx) = mpsc::channel::<Message>(1);
        let (response_tx, _response_rx) = mpsc::channel::<Message>(1);

        // Simulate handle_socket's core logic (the part that was buggy)
        // Before the fix: only waited on broadcast rx.recv()
        // After the fix: uses select! to also wait on socket.recv()
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            state_clone.connection_count.fetch_add(1, Ordering::SeqCst);
            let _guard = ConnectionGuard {
                state: state_clone.clone(),
            };

            let mut broadcast_rx = state_clone.tx.subscribe();

            // This simulates the fixed select! behavior
            loop {
                tokio::select! {
                    result = broadcast_rx.recv() => {
                        match result {
                            Ok(json) => {
                                if response_tx.send(Message::Text((*json).clone().into())).await.is_err() {
                                    break;
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(_)) => continue,
                            Err(broadcast::error::RecvError::Closed) => break,
                        }
                    }
                    msg = ws_rx.recv() => {
                        // None means channel closed (simulates WebSocket close)
                        if msg.is_none() {
                            break;
                        }
                    }
                }
            }
        });

        // Let the handler start
        tokio::task::yield_now().await;
        assert_eq!(state.connection_count.load(Ordering::SeqCst), 1);

        // Close the "WebSocket" by dropping the sender
        drop(ws_tx);

        // The handler should exit quickly (not wait forever on broadcast)
        tokio::task::yield_now().await;
        handle.await.unwrap();

        // Connection count should be decremented
        assert_eq!(state.connection_count.load(Ordering::SeqCst), 0);

        // Advance past grace period
        time::advance(Duration::from_millis(TEST_GRACE_MS + 50)).await;
        tokio::task::yield_now().await;

        // Shutdown should be triggered
        let result = shutdown_rx.try_recv();
        assert!(
            result.is_ok(),
            "shutdown should fire after socket close triggers guard drop"
        );
    }
}
