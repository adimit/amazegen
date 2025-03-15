import Maze from './Maze';
import { JSX } from 'solid-js';
import './App.scss';

export default function App(): JSX.Element {
  return (
    <>
      <div />
      <div>
        <header>
          <h1>Generate Mazes</h1>
        </header>
        <main>
          <Maze />
        </main>
      </div>
      <div />
    </>
  );
}
