import { JSX, createSignal, createEffect } from "solid-js";
import {
  configurationHashSignal as mazeSignal,
  generateMaze,
  generateMazeFromHash,
} from "./Configuration";
import { generatePdf } from "./pdfkit";

export default function Maze(): JSX.Element {
  let svgRef: HTMLDivElement | undefined;
  let input: HTMLInputElement | undefined;
  let pdfInput: HTMLInputElement | undefined;

  const {
    configuration,
    toggleFeature,
    newSeed,
    setAlgorithm,
    setSize,
    setShape,
    incrementSize,
    decrementSize,
    getSize,
    svg,
  } = mazeSignal();
  const [numberOfMazes, setNumberOfMazes] = createSignal(2);

  createEffect(() => {
    if (svgRef !== undefined) {
      svgRef.innerHTML = svg();
    }
  });

  return (
    <>
      <section>
        <h2>Size</h2>
        <button onClick={decrementSize}>-</button>
        <input
          ref={input}
          value={getSize()}
          type="number"
          onChange={(_) => {
            const n = Number(input?.value);
            if (!isNaN(n)) {
              setSize(n);
            }
          }}
        />
        <button onClick={incrementSize}>+</button>
      </section>
      <section>
        <h2>Shape</h2>
        <label>
          <input
            type="radio"
            onInput={() => setShape("Rectilinear")}
            checked={"Rectilinear" in configuration().shape}
          />
          Square
        </label>
        <label>
          <input
            type="radio"
            onInput={() => setShape("Theta")}
            checked={"Theta" in configuration().shape}
          />
          Circle
        </label>
        <label>
          <input
            type="radio"
            onInput={() => setShape("Sigma")}
            checked={"Sigma" in configuration().shape}
          />
          Hexagon
        </label>
      </section>
      <section>
        <h2>Algorithm</h2>
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
                console.error,
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
