import {
  Accessor,
  createEffect,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import { generate_maze, generate_seed, test_config } from "./pkg";

export const algorithms = ["Kruskal", "GrowingTree"] as const;
export type Algorithm = (typeof algorithms)[number];
export const features = ["Stain", "Solve"] as const;
export type Feature = (typeof features)[number];
export interface ShapeRectilinear {
  Rectilinear: [number, number];
}

console.log(test_config());

export interface ShapeTheta {
  Theta: number;
}

type KeysOfUnion<T> = T extends T ? keyof T : never;
export type ShapeKeys = KeysOfUnion<Shape>;
export type Shape = ShapeRectilinear | ShapeTheta;

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
    const n = Number(str);
    return !isNaN(n) ? n : undefined;
  };

  const parseShape = (str: string | undefined): Shape | undefined => {
    if (str === undefined) return undefined;
    const size = parseSize(str.substring(1));
    if (size !== undefined && str.startsWith("R")) {
      return { Rectilinear: [clamp(size), clamp(size)] };
    }
    if (size !== undefined && str.startsWith("T")) {
      return { Theta: clamp(size, 50) };
    }
    const legacy = parseSize(str);
    if (legacy !== undefined) {
      return { Rectilinear: [legacy, legacy] };
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

  const parse = [parseShape, parseBigint, parseAlgorithm];
  const [shape, seed, algorithm] =
    (document?.location.hash
      .substring(1)
      .split("|")
      .map((str, index) => parse[index](str)) as [
      Shape | undefined,
      bigint | undefined,
      Algorithm | undefined
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
  if ("Theta" in shape) {
    return `T${shape.Theta}`;
  }
  console.log("Error: unrecognized shape", shape);
  throw new Error(`Error: unrecognized shape ${JSON.stringify(shape)}`);
};

export const computeHash = ({
  seed,
  shape,
  algorithm,
}: Configuration): string => `${hashShape(shape)}|${seed}|${algorithm}`;

const clamp = (n: number, max: number = 100): number =>
  Math.max(2, Math.min(max, n));

export const configurationHashSignal = (): {
  configuration: Accessor<Configuration>;
  setShape: (s: ShapeKeys) => Configuration;
  setSize: (s: number) => Configuration;
  incrementSize: () => Configuration;
  decrementSize: () => Configuration;
  newSeed: () => Configuration;
  getSize: () => number;
  setAlgorithm: (a: Algorithm) => Configuration;
  addFeature: (f: Feature) => Configuration;
  removeFeature: (f: Feature) => Configuration;
  toggleFeature: (f: Feature) => Configuration;
} => {
  const [configuration, setConfiguration] = createSignal(readFromHash());

  createEffect(() => {
    if (document.location !== undefined) {
      document.location.hash = computeHash(configuration());
    }
  });

  const shapeEquals = (a: Shape, b: Shape): boolean => {
    if ("Rectilinear" in a && "Rectilinear" in b) {
      return a.Rectilinear[0] === b.Rectilinear[0];
    }
    if ("Theta" in a && "Theta" in b) {
      return a.Theta === b.Theta;
    }
    return false;
  };

  const onHashChange = (_e: HashChangeEvent): void => {
    const current = configuration();
    const hash = readFromHash();
    if (
      current.seed !== hash.seed ||
      current.algorithm !== hash.algorithm ||
      !shapeEquals(current.shape, hash.shape)
    ) {
      setConfiguration(readFromHash());
    }
  };

  onMount(() => {
    window.addEventListener("hashchange", onHashChange);
  });
  onCleanup(() => {
    window.removeEventListener("hashchange", onHashChange);
  });

  const removeFeature = (f: Feature): Configuration =>
    setConfiguration({
      ...configuration(),
      features: configuration().features.filter((of) => of !== f),
    });
  const addFeature = (f: Feature): Configuration =>
    setConfiguration({
      ...configuration(),
      features: [...new Set([...configuration().features, f])],
    });

  const adjustSize = (by: (old: number) => number): Configuration => {
    const { shape } = configuration();
    if ("Rectilinear" in shape) {
      return setConfiguration({
        ...configuration(),
        shape: {
          Rectilinear: [
            clamp(by(shape.Rectilinear[0])),
            clamp(by(shape.Rectilinear[1])),
          ],
        },
      });
    } else {
      return setConfiguration({
        ...configuration(),
        shape: {
          Theta: clamp(by(shape.Theta), 50),
        },
      });
    }
  };

  const getSize = (): number => {
    const { shape } = configuration();
    if ("Rectilinear" in shape) {
      return shape.Rectilinear[0];
    } else {
      return shape.Theta;
    }
  };

  return {
    configuration,
    setShape: (shape: ShapeKeys): Configuration =>
      setConfiguration({
        ...configuration(),
        shape:
          shape === "Rectilinear"
            ? {
                Rectilinear: [getSize() * 2, getSize() * 2],
              }
            : { Theta: Math.floor(getSize() / 2) },
      }),
    setSize: (s: number): Configuration => adjustSize(() => s),
    incrementSize: (): Configuration => adjustSize((old) => old + 1),
    decrementSize: (): Configuration => adjustSize((old) => old - 1),
    newSeed: (): Configuration =>
      setConfiguration({ ...configuration(), seed: generate_seed() }),
    setAlgorithm: (algorithm: Algorithm): Configuration =>
      setConfiguration({ ...configuration(), algorithm }),
    getSize,
    addFeature,
    removeFeature,
    toggleFeature: (f): Configuration =>
      configuration().features.includes(f) ? removeFeature(f) : addFeature(f),
  };
};
