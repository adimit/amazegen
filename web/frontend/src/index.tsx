/* @refresh reload */
import { render } from "solid-js/web";
import { lazy } from "solid-js";
import init from "./pkg/maze";

const App = lazy(async () => {
  await init();
  return await import("./App");
});

const root = document.getElementById("root");
if (root !== null) {
  render(() => <App />, root);
}
