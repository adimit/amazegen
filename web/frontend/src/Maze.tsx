import { make_svg_maze, generate_seed } from "./pkg/maze";
import { JSX, createSignal, createEffect } from "solid-js";

export default function Maze(): JSX.Element {
  let svgRef: HTMLDivElement | undefined;
  const [size, setSize] = createSignal(10);
  const seed = generate_seed();
  createEffect(() => {
    if (svgRef !== undefined) {
      svgRef.innerHTML = make_svg_maze(size(), size(), seed);
    }
  });
  let input: HTMLInputElement | undefined;
  return (
    <>
      <input
        ref={input}
        value={size()}
        onChange={(_) => {
          const n = Number(input?.value);
          if (!isNaN(n) && n > 1) {
            setSize(n);
          }
        }}
      />
      <div ref={svgRef} />
    </>
  );
}
