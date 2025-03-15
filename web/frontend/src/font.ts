import fontdata from "./assets/fonts/Bitter-Regular.ttf?url";

export const fetchFont = async (): Promise<Uint8Array> => {
  const font = await fetch(fontdata);
  const buffer = await font.arrayBuffer();
  const uint8ar = new Uint8Array(buffer);
  return uint8ar;
};
