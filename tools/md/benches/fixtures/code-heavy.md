# Code Examples

Various fenced code blocks for benchmarking syntax highlighting.

## JavaScript

```js
const express = require('express');
const app = express();

app.get('/api/users', async (req, res) => {
  const users = await db.query('SELECT * FROM users WHERE active = $1', [true]);
  const mapped = users.map(u => ({
    id: u.id,
    name: `${u.first_name} ${u.last_name}`,
    email: u.email,
    roles: u.roles.filter(r => r !== 'deprecated'),
  }));
  res.json({ data: mapped, count: mapped.length });
});

app.listen(3000, () => console.log('Server running on port 3000'));
```

## Rust

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: std::time::Instant,
}

impl<T: Clone> CacheEntry<T> {
    fn is_expired(&self) -> bool {
        std::time::Instant::now() > self.expires_at
    }
}

async fn get_or_insert<T: Clone + Send + Sync + 'static>(
    cache: &Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    key: &str,
    ttl: std::time::Duration,
    factory: impl std::future::Future<Output = T>,
) -> T {
    if let Some(entry) = cache.read().await.get(key) {
        if !entry.is_expired() {
            return entry.value.clone();
        }
    }
    let value = factory.await;
    cache.write().await.insert(key.to_string(), CacheEntry {
        value: value.clone(),
        expires_at: std::time::Instant::now() + ttl,
    });
    value
}
```

## Python

```python
import asyncio
from dataclasses import dataclass, field
from typing import Optional

@dataclass
class TreeNode:
    value: int
    left: Optional['TreeNode'] = None
    right: Optional['TreeNode'] = None
    metadata: dict = field(default_factory=dict)

    def insert(self, val: int) -> 'TreeNode':
        if val < self.value:
            if self.left is None:
                self.left = TreeNode(value=val)
            else:
                self.left.insert(val)
        elif val > self.value:
            if self.right is None:
                self.right = TreeNode(value=val)
            else:
                self.right.insert(val)
        return self

    def in_order(self) -> list[int]:
        result = []
        if self.left:
            result.extend(self.left.in_order())
        result.append(self.value)
        if self.right:
            result.extend(self.right.in_order())
        return result

async def process_tree(root: TreeNode) -> dict:
    values = root.in_order()
    await asyncio.sleep(0)  # yield point
    return {"sorted": values, "count": len(values), "min": values[0], "max": values[-1]}
```

## Go

```go
package main

import (
	"context"
	"fmt"
	"sync"
	"time"
)

type Result struct {
	Value string
	Err   error
}

func fanOut(ctx context.Context, inputs []string, workers int) <-chan Result {
	ch := make(chan Result, len(inputs))
	var wg sync.WaitGroup
	sem := make(chan struct{}, workers)

	for _, input := range inputs {
		wg.Add(1)
		sem <- struct{}{}
		go func(s string) {
			defer wg.Done()
			defer func() { <-sem }()

			select {
			case <-ctx.Done():
				ch <- Result{Err: ctx.Err()}
			default:
				time.Sleep(100 * time.Millisecond)
				ch <- Result{Value: fmt.Sprintf("processed: %s", s)}
			}
		}(input)
	}

	go func() {
		wg.Wait()
		close(ch)
	}()

	return ch
}
```

## TypeScript

```typescript
interface Config<T extends Record<string, unknown>> {
  defaults: T;
  overrides?: Partial<T>;
  validate?: (merged: T) => boolean;
}

function mergeConfig<T extends Record<string, unknown>>(config: Config<T>): T {
  const merged = { ...config.defaults, ...config.overrides };
  if (config.validate && !config.validate(merged)) {
    throw new Error('Invalid configuration after merge');
  }
  return merged;
}

type DeepReadonly<T> = {
  readonly [K in keyof T]: T[K] extends object ? DeepReadonly<T[K]> : T[K];
};

const freeze = <T extends Record<string, unknown>>(obj: T): DeepReadonly<T> =>
  Object.freeze(
    Object.fromEntries(
      Object.entries(obj).map(([k, v]) =>
        [k, typeof v === 'object' && v !== null ? freeze(v as Record<string, unknown>) : v]
      )
    )
  ) as DeepReadonly<T>;
```

## Shell

```bash
#!/usr/bin/env bash
set -euo pipefail

readonly CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/myapp"
readonly LOG_FILE="$CACHE_DIR/build.log"

setup() {
  mkdir -p "$CACHE_DIR"
  exec > >(tee -a "$LOG_FILE") 2>&1
  echo "=== Build started at $(date -Iseconds) ==="
}

build_component() {
  local name="$1"
  local src_dir="$2"
  echo "Building $name from $src_dir..."
  if [[ -f "$src_dir/Makefile" ]]; then
    make -C "$src_dir" -j"$(nproc)" 2>&1
  elif [[ -f "$src_dir/Cargo.toml" ]]; then
    cargo build --release --manifest-path "$src_dir/Cargo.toml" 2>&1
  else
    echo "No build system found for $name" >&2
    return 1
  fi
}

main() {
  setup
  local -a components=("core:src/core" "cli:src/cli" "web:src/web")
  for entry in "${components[@]}"; do
    IFS=: read -r name dir <<< "$entry"
    build_component "$name" "$dir" || { echo "FAILED: $name"; exit 1; }
  done
  echo "=== Build completed at $(date -Iseconds) ==="
}

main "$@"
```

## SQL

```sql
WITH monthly_stats AS (
  SELECT
    date_trunc('month', created_at) AS month,
    user_id,
    COUNT(*) AS order_count,
    SUM(total_amount) AS total_spent,
    AVG(total_amount) AS avg_order_value
  FROM orders
  WHERE created_at >= NOW() - INTERVAL '12 months'
    AND status NOT IN ('cancelled', 'refunded')
  GROUP BY 1, 2
),
ranked AS (
  SELECT *,
    ROW_NUMBER() OVER (PARTITION BY month ORDER BY total_spent DESC) AS rank,
    LAG(total_spent) OVER (PARTITION BY user_id ORDER BY month) AS prev_month_spent
  FROM monthly_stats
)
SELECT
  month,
  user_id,
  order_count,
  total_spent,
  avg_order_value,
  COALESCE(
    ROUND((total_spent - prev_month_spent) / NULLIF(prev_month_spent, 0) * 100, 2),
    0
  ) AS growth_pct
FROM ranked
WHERE rank <= 10
ORDER BY month DESC, rank;
```

## YAML

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-server
  labels:
    app: api-server
    version: v2
spec:
  replicas: 3
  selector:
    matchLabels:
      app: api-server
  template:
    metadata:
      labels:
        app: api-server
        version: v2
    spec:
      containers:
        - name: api
          image: registry.example.com/api:latest
          ports:
            - containerPort: 8080
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: db-credentials
                  key: url
          resources:
            requests:
              memory: "256Mi"
              cpu: "250m"
            limits:
              memory: "512Mi"
              cpu: "500m"
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 30
```

## CSS

```css
:root {
  --color-primary: #4f46e5;
  --color-surface: #ffffff;
  --radius-lg: 0.75rem;
  --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1);
}

.card {
  background: var(--color-surface);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
  padding: 1.5rem;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.card:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 15px -3px rgb(0 0 0 / 0.1);
}

@media (prefers-color-scheme: dark) {
  :root {
    --color-surface: #1e1e2e;
    --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.3);
  }
}

@container (min-width: 400px) {
  .card { display: grid; grid-template-columns: 1fr 2fr; gap: 1rem; }
}
```

## JSON

```json
{
  "name": "example-project",
  "version": "2.1.0",
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "next": "14.0.0",
    "tailwindcss": "^3.4.0"
  },
  "scripts": {
    "dev": "next dev --turbo",
    "build": "next build",
    "start": "next start",
    "lint": "next lint && prettier --check .",
    "test": "vitest run --coverage"
  }
}
```
