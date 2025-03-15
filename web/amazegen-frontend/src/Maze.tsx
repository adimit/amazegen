import { JSX, createSignal, createEffect } from 'solid-js';
import { Configuration, configurationHashSignal } from './Configuration';
import { generate_pdf } from 'amazegen';
import { saveAs } from 'file-saver';
import { fetchFont } from './font';

const getUrl = () => {
  return `${document.location.origin}${document.location.pathname}`;
};

const generatePdf = async (config: Configuration, pages: number) => {
  const printConfig: Configuration = {
    ...config,
    stroke_width: 2,
    colour: '000000',
    features: [],
  };
  const binary = generate_pdf(printConfig, pages, getUrl(), await fetchFont());
  const blob = new Blob([binary], { type: 'application/pdf' });
  saveAs(blob, 'maze.pdf');
};

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
  } = configurationHashSignal();
  const [numberOfMazes, setNumberOfMazes] = createSignal(4);

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
            onInput={() => setShape('Rectilinear')}
            checked={'Rectilinear' in configuration().shape}
          />
          Square
        </label>
        <label>
          <input
            type="radio"
            onInput={() => setShape('Theta')}
            checked={'Theta' in configuration().shape}
          />
          Circle
        </label>
        <label>
          <input
            type="radio"
            onInput={() => setShape('Sigma')}
            checked={'Sigma' in configuration().shape}
          />
          Hexagon
        </label>
      </section>
      <section>
        <h2>Algorithm</h2>
        <label>
          <input
            type="radio"
            onInput={() => setAlgorithm('GrowingTree')}
            checked={configuration().algorithm === 'GrowingTree'}
          />
          Growing Tree
        </label>
        <label>
          <input
            onInput={() => setAlgorithm('Kruskal')}
            type="radio"
            checked={configuration().algorithm === 'Kruskal'}
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
              generatePdf(configuration(), numberOfMazes());
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
            toggleFeature('Stain');
          }}
          checked={configuration().features.includes('Stain')}
        />
        Stain Maze
      </label>
      <label>
        <input
          type="checkbox"
          onInput={() => {
            toggleFeature('Solve');
          }}
          checked={configuration().features.includes('Solve')}
        />
        Show Solution
      </label>
    </>
  );
}
