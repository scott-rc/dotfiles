import { useEffect, useRef } from "preact/hooks";
import { isDiffData } from "../utils/types";
import { files, tree, branch, sourceLabel, connected, setWs, cursor, displayItems } from "../state/store";

const RECONNECT_DELAY_MS = 1000;

export function useWebSocket() {
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimer = useRef<ReturnType<typeof setTimeout>>();

  useEffect(() => {
    function connect() {
      // In dev mode, Vite proxies /ws to the Rust server.
      // In production, the Rust server serves everything.
      const protocol = location.protocol === "https:" ? "wss:" : "ws:";
      const url = `${protocol}//${location.host}/ws`;
      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        connected.value = true;
        setWs(ws);
      };

      ws.onclose = () => {
        connected.value = false;
        setWs(null);
        wsRef.current = null;
        reconnectTimer.current = setTimeout(connect, RECONNECT_DELAY_MS);
      };

      ws.onerror = () => {
        ws.close();
      };

      ws.onmessage = (event) => {
        try {
          const msg = JSON.parse(event.data);
          if (isDiffData(msg)) {
            files.value = msg.files;
            tree.value = msg.tree;
            branch.value = msg.branch;
            sourceLabel.value = msg.source_label;
            // Snap cursor to first content line
            const items = displayItems.value;
            for (let i = 0; i < items.length; i++) {
              if (items[i].type === "line") {
                cursor.value = i;
                break;
              }
            }
          }
        } catch {
          // Ignore malformed messages
        }
      };
    }

    connect();

    return () => {
      clearTimeout(reconnectTimer.current);
      wsRef.current?.close();
    };
  }, []);

  return wsRef;
}
