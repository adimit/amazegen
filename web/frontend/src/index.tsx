/* @refresh reload */
import { render } from "solid-js/web";
import { lazy } from "solid-js";
import init from "./pkg/maze";

const App = lazy(async () => {
  await init();
  return import("./App");
});

render(() => <App />, document.getElementById("root")!!);
