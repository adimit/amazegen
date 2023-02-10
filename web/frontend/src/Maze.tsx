import { JSX, createSignal, createEffect } from "solid-js";
import { configurationHashSignal, generateMaze } from "./Configuration";
import { generatePdf } from "./pdfkit";

export default function Maze(): JSX.Element {
  let svgRef: HTMLDivElement | undefined;
  let input: HTMLInputElement | undefined;
  let pdfInput: HTMLInputElement | undefined;

  const { configuration, toggleFeature, newSeed, setAlgorithm, setSize } =
    configurationHashSignal();
  const [numberOfMazes, setNumberOfMazes] = createSignal(4);

  createEffect(() => {
    if (svgRef !== undefined) {
      svgRef.innerHTML = generateMaze(configuration());
    }
  });

  return (
    <>
      <section>
        <h2>Size</h2>
        <button
          onClick={() =>
            setSize(Math.max(configuration().shape.Rectilinear[0] - 1, 2))
          }
        >
          -
        </button>
        <input
          ref={input}
          value={configuration().shape.Rectilinear[0]}
          type="number"
          onChange={(_) => {
            const n = Number(input?.value);
            if (!isNaN(n)) {
              setSize(Math.max(Math.min(n, 100), 2));
            }
          }}
        />
        <button
          onClick={() =>
            setSize(Math.min(configuration().shape.Rectilinear[0] + 1, 100))
          }
        >
          +
        </button>
      </section>
      <section>
        <h2>Type</h2>
        <label>
          <input
            type="radio"
            onInput={() => setAlgorithm("GrowingTree")}
            checked={configuration().algorithm === "GrowingTree"}
          />
          Growing Tree
        </label>
        <label>
          <input
            onInput={() => setAlgorithm("Kruskal")}
            type="radio"
            checked={configuration().algorithm === "Kruskal"}
          />
          Kruskal's
        </label>
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
          <button
            onClick={() => {
              generatePdf(configuration(), numberOfMazes()).catch(
                console.error
              );
            }}
          >
            PDF
          </button>
        </details>
      </section>
      <div ref={svgRef} />
      <button onClick={newSeed}>Refresh</button>
      <label>
        <input
          type="checkbox"
          onInput={() => {
            toggleFeature("Stain");
          }}
          checked={configuration().features.includes("Stain")}
        />
        Stain Maze
      </label>
      <label>
        <input
          type="checkbox"
          onInput={() => {
            toggleFeature("Solve");
          }}
          checked={configuration().features.includes("Solve")}
        />
        Show Solution
      </label>
    </>
  );
}
