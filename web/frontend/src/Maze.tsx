import { make_svg_maze, generate_seed } from "./pkg/maze";
import { JSX, createSignal, createEffect, Accessor } from "solid-js";

const getPreseedFromHash = (): bigint => {
  if (document?.location.hash === "") {
    return generate_seed();
  }
  try {
    return BigInt(document?.location.hash.substring(1));
  } catch (e) {
    return generate_seed();
  }
};

const seedSignal = (): {
  seed: Accessor<bigint>;
  regenerateSeed: () => void;
} => {
  const [seed, setSeed] = createSignal(getPreseedFromHash());
  createEffect(() => {
    if (document?.location !== undefined) {
      document.location.hash = seed().toString();
    }
  });

  return { seed, regenerateSeed: () => setSeed(generate_seed()) };
};

export default function Maze(): JSX.Element {
  let svgRef: HTMLDivElement | undefined;
  let input: HTMLInputElement | undefined;

  const [size, setSize] = createSignal(10);
  const { seed, regenerateSeed } = seedSignal();

  createEffect(() => {
    if (svgRef !== undefined) {
      svgRef.innerHTML = make_svg_maze(size(), size(), seed());
    }
  });

  return (
    <>
      <button onClick={() => setSize(Math.max(size() - 1, 2))}>-</button>
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
      <button onClick={() => setSize(Math.min(size() + 1, 100))}>+</button>
      <button onClick={regenerateSeed}>Refresh</button>
      <div ref={svgRef} />
    </>
  );
}
