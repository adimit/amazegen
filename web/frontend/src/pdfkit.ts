import PDF from "pdfkit";
import bitterRegular from "./assets/fonts/Bitter-Regular.otf";
import {
  Algorithm,
  computeHash,
  Configuration,
  generateMaze,
  Shape,
} from "./Configuration";
import { generate_seed } from "./pkg";

declare global {
  class PDFDocument extends PDF {}
}

const withPdf = async (
  fileName: string,
  // eslint-disable-next-line no-undef
  action: (pdf: PDFDocument) => Promise<void>
): Promise<void> => {
  /*
    Using pdfkit with Vite is a pain in the ass. pdfkit uses
    lots of Node libraries, which wouldn't be a problem on its own,
    but it also uses brfs (and something else with webpack) to bake
    in some fonts. I'd have to emulate what they're doing somehow
    in Vite with both esbuild and rollup, and I'm not keen on that.

    So instead, there's this hack. We're using a dynamic import to
    load the statically generated pdfkit file. Why are we doing that?
    Because the static file clocks in at over 1.5MB, and I don't want
    to load that on page load. BUT, it seems that rollup and esbuild
    behave differently when it comes to side effects of dynamic
    imports.

    See, esbulid just executes pdfkit.js and PDFDocument is suddenly
    available in the global scope. It's as if you had <script>-ed it.
    And you *don't* get a default export from it.

    And rollup does the exact opposite. Nothing is in the global
    scope after the import finishes, and it *does* export the main
    PDFDocument class as a default export. And I have no idea which
    magic set of options would make both behave the same.

    So we do both, and just take what's there, and tell TypeScript
    to look the other way.

    This little function neatly abstracts all the horrors, so it
    will hopefully never become a problem.
  */

  // @ts-expect-error dynamically load pdfkit and maybe get a default export…
  const { default: PDFKitExport } = await import("../pdfkit.js");
  // PDFKitExport is available with rollup, global PDFDocument with esbuild
  // eslint-disable-next-line no-undef
  const PDF = PDFKitExport ?? PDFDocument;
  const { default: blobStream } = await import("blob-stream");
  const { saveAs } = await import("file-saver");

  const pdf = new PDF();
  const stream = pdf.pipe(blobStream());
  await action(pdf);
  pdf.end();
  stream.on("finish", () => {
    const blob = stream.toBlob("application/pdf");
    saveAs(blob, `${fileName}.pdf`);
  });
};

const FRONTEND_URL = new URL("https://aleks.bg/maze");

const prettyPrintAlgorithm = (algorithm: Algorithm): string => {
  switch (algorithm) {
    case "Kruskal":
      return "Kruskal's";
    case "GrowingTree":
      return "Growing Tree (backtracking)";
  }
};

const getShapeName = (s: Shape): string => {
  if ("Rectilinear" in s) {
    return `square-${s.Rectilinear[0]}`;
  } else {
    return `circle-${s.Theta}`;
  }
};

const getSizeInformation = (s: Shape): string => {
  if ("Rectilinear" in s) {
    return `${s.Rectilinear[0]}×${s.Rectilinear[1]}`;
  } else {
    return `${s.Theta}`;
  }
};

export const generatePdf = async (
  configuration: Configuration,
  numberOfMazes: number
): Promise<void> => {
  const { default: SVGtoPDF } = await import("svg-to-pdfkit");
  const QR = await import("qrcode");
  const font = await (await fetch(bitterRegular)).arrayBuffer();
  await withPdf(`maze-${getShapeName(configuration.shape)}`, async (pdf) => {
    const addMaze = async (mazeSeed: bigint): Promise<void> => {
      const myConf = {
        ...configuration,
        seed: mazeSeed,
        colour: "000000",
        stroke_width: 2,
      };
      const qr = await QR.toString(
        new URL(`#${computeHash(myConf)}`, FRONTEND_URL).toString(),
        {
          type: "svg",
          errorCorrectionLevel: "high",
        }
      );

      const template = document.createElement("template");

      const svg = generateMaze(myConf);
      template.innerHTML = svg;
      const svgNode = template.content.firstChild as SVGElement;
      const width = document.createAttribute("width");
      width.value = "680px";
      svgNode.attributes.setNamedItem(width);
      const height = document.createAttribute("height");
      height.value = "680px";
      svgNode.attributes.setNamedItem(height);
      pdf
        .font(font)
        .text(
          `Size: ${getSizeInformation(
            myConf.shape
          )}\nSeed: ${mazeSeed}\nAlgorithm: ${prettyPrintAlgorithm(
            myConf.algorithm
          )}`,
          50,
          600
        );
      SVGtoPDF(pdf, template.innerHTML, 50, 50);
      SVGtoPDF(pdf, qr, 487, 240, {
        width: 80,
      });
    };
    await addMaze(configuration.seed);
    for (let i = 1; i < numberOfMazes; i++) {
      pdf.addPage();
      await addMaze(generate_seed());
    }
  });
};
