import { make_svg_maze, generate_seed } from "./pkg";
import {
  JSX,
  createSignal,
  createEffect,
  Accessor,
  onCleanup,
  onMount,
  batch,
} from "solid-js";
import { withPdf } from "./pdfkit";
const DEFAULT_MAZE_SIZE = 10;
const FRONTEND_URL = new URL("https://aleks.bg/maze");

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

const computeHash = ({ seed, size }: MazeParameters) => `${size}|${seed}`;

const writeToHash = (params: MazeParameters) => {
  if (document.location) {
    document.location.hash = computeHash(params);
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

  const onHashChange = (_e: HashChangeEvent) => {
    const { seed: newSeed, size: newSize } = readFromHash();
    batch(() => {
      setSeed(newSeed);
      setSize(newSize);
    });
  };

  onMount(() => window.addEventListener("hashchange", onHashChange));
  onCleanup(() => window.removeEventListener("hashchange", onHashChange));

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
  const [showSolution, setShowSolution] = createSignal(false);
  const [stainMaze, setStainMaze] = createSignal(false);

  createEffect(() => {
    if (svgRef !== undefined) {
      svgRef.innerHTML = make_svg_maze(
        size(),
        size(),
        seed(),
        "eeeeee",
        stainMaze(),
        showSolution()
      );
    }
  });

  const pdf = async () => {
    const { default: SVGtoPDF } = await import("svg-to-pdfkit");
    const QR = await import("qrcode");
    withPdf(`maze-${size()}`, async (pdf) => {
      const addMaze = async (mazeSeed: bigint) => {
        const qr = await QR.toString(
          new URL(
            `#${computeHash({ seed: mazeSeed, size: size() })}`,
            FRONTEND_URL
          ).toString(),
          {
            type: "svg",
            errorCorrectionLevel: "high",
          }
        );

        const template = document.createElement("template");
        const svg = make_svg_maze(
          size(),
          size(),
          mazeSeed,
          "000000",
          false,
          false
        );
        template.innerHTML = svg;
        const svgNode = template.content.firstChild as SVGElement;
        svgNode.attributes.getNamedItem("width")!!.value = "680px";
        svgNode.attributes.getNamedItem("height")!!.value = "680px";
        SVGtoPDF(pdf, template.innerHTML, 50, 50);
        SVGtoPDF(pdf, qr, 487, 220, {
          width: 80,
        });
      };
      await addMaze(seed());
      for (var i = 1; i < numberOfMazes(); i++) {
        pdf.addPage();
        await addMaze(generate_seed());
      }
    });
  };

  return (
    <>
      <h2>Size</h2>
      <section>
        <button onClick={() => setSize(Math.max(size() - 1, 2))}>-</button>
        <input
          ref={input}
          value={size()}
          type="number"
          onChange={(_) => {
            const n = Number(input?.value);
            if (!isNaN(n)) {
              setSize(Math.max(Math.min(n, 100), 2));
            }
          }}
        />
        <button onClick={() => setSize(Math.min(size() + 1, 100))}>+</button>
      </section>
      <section>
        <details>
          <summary>Print</summary>
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
          pages
          <button onClick={pdf}>PDF</button>
        </details>
      </section>
      <div ref={svgRef} />
      <button onClick={regenerateSeed}>Refresh</button>
      <label>
        <input
          type="checkbox"
          onInput={() => setStainMaze(!stainMaze())}
          checked={stainMaze()}
        />
        Stain Maze
      </label>
      <label>
        <input
          type="checkbox"
          onInput={() => setShowSolution(!showSolution())}
          checked={showSolution()}
        />
        Show Solution
      </label>
    </>
  );
}
