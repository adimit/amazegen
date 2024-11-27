import {
  Accessor,
  createEffect,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import { generate_maze, generate_seed } from "./pkg";
import { generate_maze_from_hash } from "./pkg/amazegen";

export const algorithms = ["Kruskal", "GrowingTree"] as const;
export type Algorithm = (typeof algorithms)[number];
export const features = ["Stain", "Solve"] as const;
export type Feature = (typeof features)[number];

export interface ShapeRectilinear {
  Rectilinear: [number, number];
}

export interface ShapeTheta {
  Theta: number;
}

export interface ShapeSigma {
  Sigma: number;
}

type KeysOfUnion<T> = T extends T ? keyof T : never;
export type ShapeKeys = KeysOfUnion<Shape>;
export type Shape = ShapeRectilinear | ShapeTheta | ShapeSigma;

export interface Configuration {
  algorithm: Algorithm;
  colour: string;
  features: Feature[];
  seed: bigint;
  shape: Shape;
  stroke_width: number;
}

export type SVG = string;

// there's no type for the Rust import, so we make one here
// eslint-disable-next-line @typescript-eslint/naming-convention
export const generateMaze: (config: Configuration) => SVG = generate_maze;

export const generateMazeFromHash: (request: WebRequest) => {
  svg: SVG;
  configuration: Configuration;
} = (request) => {
  const result = generate_maze_from_hash(request);
  if (result) {
    return result;
  }
  throw new Error("Failed to generate maze from hash, check console.");
};

export const DEFAULT_MAZE_SIZE = 10;

const readFromHash = (): Configuration => {
  const getDefaultConfiguration = (): Configuration => ({
    seed: generate_seed(),
    algorithm: "GrowingTree",
    shape: { Rectilinear: [DEFAULT_MAZE_SIZE, DEFAULT_MAZE_SIZE] },
    features: [],
    colour: "EEEEEE",
    stroke_width: 8,
  });

  const parseSize = (str: string | undefined): number | undefined => {
    if (str === undefined || str === "") return undefined;
    const n = Number(str);
    return !isNaN(n) ? n : undefined;
  };

  const parseShape = (str: string | undefined): Shape | undefined => {
    if (str === undefined) return undefined;
    const size = parseSize(str.substring(1));
    if (size !== undefined && str.startsWith("R")) {
      return rect(size);
    }
    if (size !== undefined && str.startsWith("T")) {
      return theta(size);
    }
    if (size !== undefined && str.startsWith("S")) {
      return sigma(size);
    }
    const legacy = parseSize(str);
    if (legacy !== undefined) {
      return rect(legacy);
    }
    return undefined;
  };

  const parseBigint = (str: string | undefined): bigint | undefined => {
    if (str === undefined) return undefined;
    try {
      return BigInt(str);
    } catch (e) {
      return undefined;
    }
  };

  const parseAlgorithm = (str: string | undefined): Algorithm | undefined => {
    if (algorithms.includes(str as Algorithm)) {
      return str as Algorithm;
    }
    return undefined;
  };

  const parse = [parseShape, parseAlgorithm, parseBigint];
  const [shape, algorithm, seed] =
    (document?.location.hash
      .substring(1)
      .split("|")
      .map((str, index) => parse[index](str)) as [
      Shape | undefined,
      Algorithm | undefined,
      bigint | undefined,
    ]) ?? [];

  return {
    ...getDefaultConfiguration(),
    ...(shape !== undefined && { shape }),
    ...(seed !== undefined && { seed }),
    ...(algorithm !== undefined && { algorithm }),
  };
};

const hashShape = (shape: Shape): string => {
  if ("Rectilinear" in shape) {
    return `R${shape.Rectilinear[0]}`;
  }
  if ("Sigma" in shape) {
    return `S${shape.Sigma}`;
  }
  return `T${shape.Theta}`;
};

export const computeHash = ({
  seed,
  shape,
  algorithm,
}: Configuration): string => `${hashShape(shape)}|${algorithm}|${seed}`;

interface WebRequest {
  hash: string;
  colour: string;
  features: Feature[];
}
const getInitialParameters = (): WebRequest => ({
  hash: window.location.hash.substring(1),
  colour: "EEEEEE",
  features: [],
});

export const configurationHashSignal = (): {
  configuration: Accessor<Configuration>;
  setShape: (s: ShapeKeys) => Configuration;
  setSize: (s: number) => Configuration;
  incrementSize: () => Configuration;
  decrementSize: () => Configuration;
  newSeed: () => Configuration;
  getSize: () => number;
  setAlgorithm: (a: Algorithm) => WebRequest;
  addFeature: (f: Feature) => WebRequest;
  removeFeature: (f: Feature) => WebRequest;
  toggleFeature: (f: Feature) => WebRequest;
  svg: SVG;
} => {
  const [configuration2, setConfiguration] = createSignal(readFromHash());
  const [params, setParams] = createSignal(getInitialParameters());
  const { configuration, svg } = generateMazeFromHash(params());

  createEffect(() => {
    if (document.location !== undefined) {
      document.location.replace(`#${computeHash(configuration)}`);
    }
  });

  const onHashChange = (_e: HashChangeEvent): void => {
    setParams({
      ...params(),
      hash: window.location.hash.substring(1),
    });
  };

  onMount(() => {
    window.addEventListener("hashchange", onHashChange);
  });
  onCleanup(() => {
    window.removeEventListener("hashchange", onHashChange);
  });

  const removeFeature = (f: Feature): WebRequest =>
    setParams({
      ...params(),
      features: params().features.filter((of) => of !== f),
    });
  const addFeature = (f: Feature): WebRequest =>
    setParams({
      ...params(),
      features: [...new Set([...params().features, f])],
    });

  const adjustSize = (by: (old: number) => number): Configuration => {
    const { shape } = configuration2();
    if ("Rectilinear" in shape) {
      return setConfiguration({
        ...configuration2(),
        shape: rect(by(shape.Rectilinear[0])),
      });
    } else if ("Theta" in shape) {
      return setConfiguration({
        ...configuration2(),
        shape: theta(by(shape.Theta)),
      });
    } else {
      return setConfiguration({
        ...configuration2(),
        shape: sigma(by(shape.Sigma)),
      });
    }
  };

  const getSize = (): number => {
    const { shape } = configuration2();
    if ("Rectilinear" in shape) {
      return shape.Rectilinear[0];
    } else if ("Theta" in shape) {
      return shape.Theta;
    } else {
      return shape.Sigma;
    }
  };

  const adjustSizeToNewShape = (newShape: ShapeKeys) => {
    const currentShape = configuration2().shape;
    if ("Theta" in currentShape && newShape !== "Theta") {
      return getSize() * 2;
    }
    if (newShape === "Theta") {
      return Math.floor(getSize() / 2);
    }
    return getSize();
  };
  const setShape = (shape: ShapeKeys): Shape => {
    const size = adjustSizeToNewShape(shape);
    switch (shape) {
      case "Rectilinear":
        return rect(size);
      case "Theta":
        return theta(size);
      case "Sigma":
        return sigma(size);
    }
  };

  return {
    configuration: configuration2,
    setShape: (shape: ShapeKeys): Configuration =>
      setConfiguration({
        ...configuration2(),
        shape: setShape(shape),
      }),
    setSize: (s: number): Configuration => adjustSize(() => s),
    incrementSize: (): Configuration => adjustSize((old) => old + 1),
    decrementSize: (): Configuration => adjustSize((old) => old - 1),
    newSeed: (): Configuration =>
      setConfiguration({ ...configuration2(), seed: generate_seed() }),
    setAlgorithm: (algorithm: Algorithm): Configuration =>
      setConfiguration({ ...configuration2(), algorithm }),
    getSize,
    addFeature,
    removeFeature,
    toggleFeature: (f): WebRequest =>
      params().features.includes(f) ? removeFeature(f) : addFeature(f),
    svg,
  };
};

const clamp = (n: number, max: number): number =>
  Math.floor(Math.max(2, Math.min(max, n)));
const rect = (size: number): Shape => ({
  Rectilinear: [clamp(size, 100), clamp(size, 100)],
});
const theta = (size: number): Shape => ({ Theta: clamp(size, 50) });
const sigma = (size: number): Shape => ({ Sigma: clamp(size, 100) });
