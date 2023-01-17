import {make_svg_maze, generate_seed} from './pkg/maze';
import { createSignal, createEffect } from 'solid-js';

export default function Maze() {
  let svgRef: HTMLDivElement | undefined;
  const [size, setSize] = createSignal(10);
  const seed = generate_seed();
  createEffect(() => {
    if (svgRef) {
      svgRef.innerHTML = make_svg_maze(size(), size(), seed);
    }
  });
  let input: HTMLInputElement | undefined;
  return (<>
    <input ref={input} value={size()} onChange={(_) => {
      const n = Number(input?.value);
      if (n !== NaN && n > 1) {
        setSize(n);
      }
    }} />
    <div ref={svgRef} />
  </>)
}

