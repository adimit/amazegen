import {
  Accessor,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
  untrack,
} from "solid-js";
import { generate_maze, generate_seed } from "./pkg";

export const algorithms = ["Kruskal", "GrowingTree"] as const;
export type Algorithm = (typeof algorithms)[number];
export const features = ["Stain", "Solve"] as const;
export type Feature = (typeof features)[number];
export interface ShapeRectilinear {
  Rectilinear: [number, number];
}

export type Shape = ShapeRectilinear;

export interface Configuration {
  algorithm: Algorithm;
  colour: string;
  features: Feature[];
  seed: bigint;
  shape: Shape;
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
  });

  const parseSize = (str: string | undefined): number | undefined => {
    const n = Number(str);
    return !isNaN(n) && n > 1 && n < 101 ? n : undefined;
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

  const parse = [parseSize, parseBigint, parseAlgorithm];
  const [size, seed, algorithm] =
    (document?.location.hash
      .substring(1)
      .split("|")
      .map((str, index) => parse[index](str)) as [
      number | undefined,
      bigint | undefined,
      Algorithm | undefined
    ]) ?? [];

  return {
    ...getDefaultConfiguration(),
    ...(size !== undefined && {
      shape: { Rectilinear: [size, size] },
    }),
    ...(seed !== undefined && { seed }),
    ...(algorithm !== undefined && { algorithm }),
  };
};

export const computeHash = ({
  seed,
  shape: {
    Rectilinear: [size, _],
  },
  algorithm,
}: Configuration): string => `${size}|${seed}|${algorithm}`;

export const configurationHashSignal = (): {
  configuration: Accessor<Configuration>;
  setSize: (s: number) => Configuration;
  newSeed: () => Configuration;
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

  const onHashChange = (_e: HashChangeEvent): void => {
    const current = configuration();
    const hash = readFromHash();
    if (
      current.seed !== hash.seed ||
      current.algorithm !== hash.algorithm ||
      current.shape.Rectilinear[0] !== hash.shape.Rectilinear[0]
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

  return {
    configuration,
    setSize: (s: number): Configuration =>
      setConfiguration({ ...configuration(), shape: { Rectilinear: [s, s] } }),
    newSeed: (): Configuration =>
      setConfiguration({ ...configuration(), seed: generate_seed() }),
    setAlgorithm: (algorithm: Algorithm): Configuration =>
      setConfiguration({ ...configuration(), algorithm }),
    addFeature,
    removeFeature,
    toggleFeature: (f): Configuration =>
      configuration().features.includes(f) ? removeFeature(f) : addFeature(f),
  };
};
