import init from './pkg/maze';
import {lazy} from 'solid-js';

const LazyMaze = lazy(async () => {
  await init();
  return import("./Maze");
})

function App() {
   return (
    <div>
      <LazyMaze />
    </div>
  );
}

export default App;
