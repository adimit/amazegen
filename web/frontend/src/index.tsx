/* @refresh reload */
import { render } from "solid-js/web";
import { lazy, Suspense } from "solid-js";
import init from "./pkg/maze";

const App = lazy(async () => {
  await init();
  return await import("./App");
});

const root = document.getElementById("root");
if (root !== null) {
  render(
    () => (
      <Suspense fallback={<span>Waiting for WASM</span>}>
        <App />{" "}
      </Suspense>
    ),
    root
  );
}
