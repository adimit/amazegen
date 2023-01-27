import Maze from "./Maze";
import { JSX } from "solid-js";
import "./App.scss";

export default function App(): JSX.Element {
  return (
    <>
      <header>
        <h1>A maze generator</h1>
      </header>
      <main>
        <Maze />
      </main>
    </>
  );
}
