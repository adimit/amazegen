/* @refresh reload */
import { render } from "solid-js/web";
import { lazy, Suspense } from "solid-js";

const App = lazy(async () => {
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
