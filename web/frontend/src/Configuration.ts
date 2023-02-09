import { generate_maze as untyped_generate_maze } from "./pkg";

export const algorithms = ["Kruskal", "GrowingTree"] as const;
export type Algorithm = (typeof algorithms)[number];
export const features = ["Stain", "Solve"] as const;
export type Feature = (typeof features)[number];
export type Shape = { Rectilinear: [number, number] };

export type Configuration = {
  algorithm: Algorithm;
  colour: string;
  features: Feature[];
  seed: bigint;
  shape: Shape;
};

export type SVG = string;

export const generate_maze: (config: Configuration) => SVG =
  untyped_generate_maze;
