import { render } from "preact";
import "./index.css";
import "./utils/perf";
import { App } from "./components/App";

render(<App />, document.getElementById("app")!);
