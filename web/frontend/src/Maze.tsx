import { make_svg_maze, generate_seed } from "./pkg/maze";
import { JSX, createSignal, createEffect, Accessor } from "solid-js";
import { withPdf } from "./pdfkit";
const DEFAULT_MAZE_SIZE = 10;

interface MazeParameters {
  size: number;
  seed: bigint;
}

const getDefaultMazeParameters = (): MazeParameters => ({
  seed: generate_seed(),
  size: DEFAULT_MAZE_SIZE,
});

const readFromHash = (): MazeParameters => {
  if (document?.location.hash === "" || document?.location.hash === undefined) {
    return getDefaultMazeParameters();
  } else {
    const [sizeStr, seedStr] = document.location.hash.substring(1).split("|");
    const size = Number(sizeStr);
    if (isNaN(size)) {
      return getDefaultMazeParameters();
    }
    try {
      return {
        size,
        seed: BigInt(seedStr),
      };
    } catch (e) {
      return getDefaultMazeParameters();
    }
  }
};

const writeToHash = ({ seed, size }: MazeParameters) => {
  if (document.location) {
    document.location.hash = `${size}|${seed}`;
  }
};

const parameterSignal = (): {
  seed: Accessor<bigint>;
  size: Accessor<number>;
  regenerateSeed: () => void;
  setSize: (newSize: number) => void;
} => {
  const params = readFromHash();
  const [seed, setSeed] = createSignal(params.seed);
  const [size, setSize] = createSignal(params.size);
  createEffect(() => {
    writeToHash({
      seed: seed(),
      size: size(),
    });
  });

  return {
    size,
    setSize,
    seed,
    regenerateSeed: () => setSeed(generate_seed()),
  };
};

export default function Maze(): JSX.Element {
  let svgRef: HTMLDivElement | undefined;
  let input: HTMLInputElement | undefined;
  let pdfInput: HTMLInputElement | undefined;

  const { seed, size, setSize, regenerateSeed } = parameterSignal();
  const [numberOfMazes, setNumberOfMazes] = createSignal(4);

  createEffect(() => {
    if (svgRef !== undefined) {
      svgRef.innerHTML = make_svg_maze(size(), size(), seed(), "eeeeee");
    }
  });

  const pdf = async () => {
    const { default: SVGtoPDF } = await import("svg-to-pdfkit");
    withPdf(`maze-${size()}`, (pdf) => {
      const addMaze = (mazeSeed: bigint) => {
        const template = document.createElement("template");
        const svg = make_svg_maze(size(), size(), mazeSeed, "000000");
        template.innerHTML = svg;
        const svgNode = template.content.firstChild as SVGElement;
        svgNode.attributes.getNamedItem("width")!!.value = "680px";
        svgNode.attributes.getNamedItem("height")!!.value = "680px";
        SVGtoPDF(pdf, template.innerHTML, 50, 50);
      };
      addMaze(seed());
      for (var i = 1; i < numberOfMazes(); i++) {
        pdf.addPage();
        addMaze(generate_seed());
      }
    });
  };

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
      <input
        ref={pdfInput}
        value={numberOfMazes()}
        onChange={(_) => {
          const n = Number(pdfInput?.value);
          if (!isNaN(n) && n > 0) {
            setNumberOfMazes(n);
          }
        }}
      />
      <button onClick={pdf}>PDF</button>
      <div ref={svgRef} />
    </>
  );
}
